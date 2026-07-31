//! # bifrost — Merkle-DAG audit chain + WORM_FS
//!
//! Every cap transfer and JIT compile in snap-os is sealed as an unforgeable,
//! content-addressed event.  The chain is append-only, cryptographically linked,
//! and enforced by filesystem immutability.
//!
//! ## Quick start
//! ```no_run
//! use bifrost::seal::Bifrost;
//! use bifrost::verify::verify_chain;
//! use bifrost::event::Cid;
//!
//! let mut chain = Bifrost::open("./my_chain").unwrap();
//! chain.seal_genesis().unwrap();
//!
//! let policy = chain.worm.seal_blob(b"%cap_transfer :- true.", None).unwrap();
//! let cid = chain.seal_cap_transfer(
//!     Cid::of(b"soul_a"), Cid::of(b"soul_b"), Cid::of(b"cap"), policy
//! ).unwrap();
//!
//! let head  = chain.dag.head().unwrap().unwrap();
//! let report = verify_chain(&head, &chain.dag, &chain.worm).unwrap();
//! assert!(report.ok);
//! ```

pub mod append;
pub mod dag;
pub mod error;
pub mod event;
pub mod seal;
pub mod verify;
pub mod worm;
