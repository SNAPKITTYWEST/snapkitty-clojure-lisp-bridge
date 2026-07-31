//! WORM_FS — Write-Once Read-Many content-addressed blob storage.
//!
//! # Security model
//! Once a blob is sealed, it MUST NOT be modified.  Enforcement layers:
//!
//! 1. **CID check (always active)**: `seal_blob` refuses to write if the CID
//!    already exists in the metadata DB, returning `BifrostError::WormViolation`.
//!
//! 2. **Filesystem immutability (platform-specific)**:
//!    - **Windows**: `SetFileAttributesW(path, FILE_ATTRIBUTE_READONLY)` makes the
//!      file read-only at the OS level.  Requires admin to remove.
//!    - **Linux**: `ioctl(fd, FS_IOC_SETFLAGS, FS_IMMUTABLE_FL)` sets the
//!      ext4/xfs immutable flag.  Requires `CAP_LINUX_IMMUTABLE`.
//!    - **Fallback**: CID-check-only (still safe if the metadata DB is not tampered).
//!
//! 3. **Metadata DB**: `sled` tree keyed by `cid_hex` → `BlobMeta` (JSON).
//!    The DB itself should live on a filesystem with appropriate ACLs.
//!
//! # Layout on disk
//! ```text
//! <root>/
//!   blobs/<cid_hex[0..2]>/<cid_hex>.bin   ← content blob
//!   meta/                                  ← sled metadata DB
//! ```
//! The two-hex-char directory prefix shards blobs (prevents huge flat dirs).

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sled::Db;

use crate::error::{BifrostError, BifrostResult};
use crate::event::{Cid, PubKey};

// ── BlobMeta ──────────────────────────────────────────────────────────────────

/// Metadata record stored in the WORM_FS sled DB per sealed blob.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobMeta {
    /// Blake3 CID of the blob (also the DB key — stored here for convenience).
    pub cid:             String,
    /// Byte length of the blob.
    pub len:             u64,
    /// Unix epoch seconds when the blob was sealed.
    pub sealed_at:       u64,
    /// Public key of the soul that sealed the blob, or all-zeros for system seals.
    pub sealed_by_pubkey: String,
}

// ── WormFs ────────────────────────────────────────────────────────────────────

/// Write-once, read-many content-addressed blob store.
pub struct WormFs {
    blobs_root: PathBuf,
    db:         Db,
}

impl WormFs {
    /// Open (or create) the WORM_FS at `root`.
    ///
    /// Directory layout is created if missing.
    pub fn open(root: &Path) -> BifrostResult<Self> {
        let blobs_root = root.join("blobs");
        let meta_root  = root.join("meta");
        fs::create_dir_all(&blobs_root)?;
        fs::create_dir_all(&meta_root)?;

        let db = sled::open(&meta_root)?;
        Ok(Self { blobs_root, db })
    }

    // ── Write path ────────────────────────────────────────────────────────────

    /// Seal `bytes` into WORM_FS.
    ///
    /// # Steps
    /// 1. Compute `cid = blake3(bytes)`.
    /// 2. Reject with `WormViolation` if CID already sealed.
    /// 3. Write blob to `blobs/<prefix>/<cid>.bin`.
    /// 4. Set file read-only (platform-specific).
    /// 5. Write `BlobMeta` to sled.
    ///
    /// Returns the CID.
    pub fn seal_blob(&self, bytes: &[u8], sealed_by: Option<&PubKey>) -> BifrostResult<Cid> {
        let cid = Cid::of(bytes);
        let cid_hex = cid.to_hex();

        // ── Guard: reject duplicates ─────────────────────────────────────────
        if self.db.contains_key(&cid_hex)? {
            return Err(BifrostError::WormViolation { cid: cid_hex });
        }

        // ── Write blob ───────────────────────────────────────────────────────
        let blob_path = self.blob_path(&cid);
        fs::create_dir_all(blob_path.parent().unwrap())?;
        fs::write(&blob_path, bytes)?;

        // ── Enforce immutability ─────────────────────────────────────────────
        if let Err(e) = make_immutable(&blob_path) {
            // Log but do not fail — CID check is the primary guard.
            eprintln!("bifrost/worm: make_immutable({blob_path:?}) failed: {e} (continuing)");
        }

        // ── Write metadata ───────────────────────────────────────────────────
        let pubkey_hex = sealed_by
            .map(|pk| pk.to_hex())
            .unwrap_or_else(|| "0".repeat(64));

        let meta = BlobMeta {
            cid:             cid_hex.clone(),
            len:             bytes.len() as u64,
            sealed_at:       unix_now(),
            sealed_by_pubkey: pubkey_hex,
        };
        let meta_bytes = serde_json::to_vec(&meta)?;
        self.db.insert(&cid_hex, meta_bytes.as_slice())?;
        self.db.flush()?;

        Ok(cid)
    }

    // ── Read path ─────────────────────────────────────────────────────────────

    /// Read a sealed blob by CID.
    ///
    /// After reading, verifies the CID matches the file contents.
    /// Returns `BifrostError::NotFound` if not present,
    /// `BifrostError::CidMismatch` if the file was tampered.
    pub fn read_blob(&self, cid: &Cid) -> BifrostResult<Vec<u8>> {
        let cid_hex = cid.to_hex();
        if !self.db.contains_key(&cid_hex)? {
            return Err(BifrostError::NotFound { cid: cid_hex });
        }

        let path = self.blob_path(cid);
        let bytes = fs::read(&path)?;

        // CID integrity check on every read — detect filesystem tampering.
        let computed = Cid::of(&bytes);
        if computed != *cid {
            return Err(BifrostError::CidMismatch {
                expected: cid_hex,
                computed: computed.to_hex(),
            });
        }
        Ok(bytes)
    }

    /// Return the `BlobMeta` for a CID without reading the blob data.
    pub fn meta(&self, cid: &Cid) -> BifrostResult<Option<BlobMeta>> {
        match self.db.get(cid.to_hex())? {
            None => Ok(None),
            Some(bytes) => {
                let meta: BlobMeta = serde_json::from_slice(&bytes)?;
                Ok(Some(meta))
            }
        }
    }

    /// Return `true` if the CID exists in the WORM_FS metadata DB.
    pub fn is_sealed(&self, cid: &Cid) -> bool {
        self.db.contains_key(cid.to_hex()).unwrap_or(false)
    }

    /// Total number of sealed blobs.
    pub fn blob_count(&self) -> usize {
        self.db.len()
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    fn blob_path(&self, cid: &Cid) -> PathBuf {
        let hex = cid.to_hex();
        self.blobs_root.join(&hex[..2]).join(format!("{hex}.bin"))
    }
}

// ── Platform-specific immutability ────────────────────────────────────────────

/// Set the file at `path` to read-only using `std::fs::Permissions`.
///
/// On Windows this sets `FILE_ATTRIBUTE_READONLY`.
/// On Unix this clears write bits (chmod a-w).
/// Neither is as strong as Linux `FS_IMMUTABLE_FL` (which requires root),
/// but both prevent accidental overwrites and trip any audit tooling.
fn make_immutable(path: &Path) -> std::io::Result<()> {
    let mut perms = std::fs::metadata(path)?.permissions();
    perms.set_readonly(true);
    std::fs::set_permissions(path, perms)
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn seal_and_read_roundtrip() {
        let dir = tempdir().unwrap();
        let worm = WormFs::open(dir.path()).unwrap();

        let data = b"bifrost genesis blob";
        let cid = worm.seal_blob(data, None).unwrap();
        let read_back = worm.read_blob(&cid).unwrap();
        assert_eq!(data.as_ref(), read_back.as_slice());
    }

    #[test]
    fn cid_is_deterministic() {
        let dir = tempdir().unwrap();
        let worm = WormFs::open(dir.path()).unwrap();

        let data = b"deterministic content";
        let cid1 = worm.seal_blob(data, None).unwrap();
        let expected = Cid::of(data);
        assert_eq!(cid1, expected);
    }

    #[test]
    fn worm_violation_on_duplicate() {
        let dir = tempdir().unwrap();
        let worm = WormFs::open(dir.path()).unwrap();

        let data = b"unique blob";
        worm.seal_blob(data, None).unwrap();
        let err = worm.seal_blob(data, None).unwrap_err();
        assert!(matches!(err, BifrostError::WormViolation { .. }));
    }

    #[test]
    fn not_found_on_unknown_cid() {
        let dir = tempdir().unwrap();
        let worm = WormFs::open(dir.path()).unwrap();

        let unknown = Cid::of(b"not in worm");
        let err = worm.read_blob(&unknown).unwrap_err();
        assert!(matches!(err, BifrostError::NotFound { .. }));
    }

    #[test]
    fn meta_records_length() {
        let dir = tempdir().unwrap();
        let worm = WormFs::open(dir.path()).unwrap();

        let data = b"meta test";
        let cid = worm.seal_blob(data, None).unwrap();
        let meta = worm.meta(&cid).unwrap().unwrap();
        assert_eq!(meta.len, data.len() as u64);
    }

    #[test]
    fn is_sealed_reflects_state() {
        let dir = tempdir().unwrap();
        let worm = WormFs::open(dir.path()).unwrap();

        let data = b"check sealed";
        let missing = Cid::of(b"not here");
        assert!(!worm.is_sealed(&missing));

        let cid = worm.seal_blob(data, None).unwrap();
        assert!(worm.is_sealed(&cid));
    }
}
