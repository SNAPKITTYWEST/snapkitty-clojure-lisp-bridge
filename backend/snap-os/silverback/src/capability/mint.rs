use super::cnode::{Cap, CapError, CapPtr, CapType, CSpace};
use super::rights::Rights;

pub struct MintArgs {
    pub cap_type:  CapType,
    pub object_id: u64,
    pub rights:    Rights,
    pub badge:     u64,
}

/// Derive a new capability from `parent`, restricting to `args.rights`.
/// The parent must have DELEGATE rights; derived rights are never amplified.
pub fn mint(parent: &Cap, args: MintArgs) -> Result<Cap, CapError> {
    if parent.is_null() {
        return Err(CapError::NullCap);
    }
    if parent.sealed {
        return Err(CapError::AlreadySealed);
    }
    if !parent.rights.can_delegate() {
        return Err(CapError::InsufficientRights);
    }
    Ok(Cap {
        cap_type:  args.cap_type,
        object_id: args.object_id,
        rights:    args.rights.restrict(parent.rights),
        badge:     args.badge,
        sealed:    false,
    })
}

/// Copy src_space[src] → dst_space[dst], masking rights.
pub fn copy<const N: usize, const M: usize>(
    src_space: &CSpace<N>,
    src: CapPtr,
    dst_space: &mut CSpace<M>,
    dst: CapPtr,
    rights_mask: Rights,
) -> Result<(), CapError> {
    let src_cap = *src_space.lookup(src)?;
    if src_cap.is_null() {
        return Err(CapError::NullCap);
    }
    let new_cap = Cap {
        cap_type:  src_cap.cap_type,
        object_id: src_cap.object_id,
        rights:    src_cap.rights.restrict(rights_mask),
        badge:     src_cap.badge,
        sealed:    false,
    };
    dst_space.insert(dst, new_cap)
}

/// Move src_space[src] → dst_space[dst], deleting the source slot.
/// If the destination insert fails, the source slot is restored.
pub fn transfer<const N: usize>(
    space: &mut CSpace<N>,
    src: CapPtr,
    dst: CapPtr,
) -> Result<(), CapError> {
    let cap = space.delete(src)?;
    space.insert(dst, cap).map_err(|e| {
        let _ = space.insert(src, cap);
        e
    })
}

/// Zero a capability slot.
pub fn revoke<const N: usize>(space: &mut CSpace<N>, ptr: CapPtr) -> Result<(), CapError> {
    space.delete(ptr).map(|_| ())
}

/// Seal a capability: strips DELEGATE and WRITE permanently.
/// A sealed cap can only be invoked or read — it cannot be shared further.
pub fn seal<const N: usize>(space: &mut CSpace<N>, ptr: CapPtr) -> Result<(), CapError> {
    let existing = *space.lookup(ptr)?;
    if existing.is_null() {
        return Err(CapError::NullCap);
    }
    if existing.sealed {
        return Err(CapError::AlreadySealed);
    }
    space.delete(ptr)?;
    space.insert(ptr, Cap {
        cap_type:  existing.cap_type,
        object_id: existing.object_id,
        rights:    existing.rights.restrict(Rights::READ | Rights::INVOKE),
        badge:     existing.badge,
        sealed:    true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::cnode::CapType;

    fn ep(rights: Rights) -> Cap {
        Cap { cap_type: CapType::Endpoint, object_id: 1, rights, badge: 0, sealed: false }
    }

    #[test]
    fn mint_restricts_rights() {
        let parent = ep(Rights::ALL);
        let child = mint(&parent, MintArgs {
            cap_type: CapType::Endpoint,
            object_id: 1,
            rights: Rights::READ | Rights::INVOKE,
            badge: 0,
        }).unwrap();
        assert!(child.rights.is_read());
        assert!(child.rights.is_invoke());
        assert!(!child.rights.can_delegate());
    }

    #[test]
    fn mint_blocked_without_delegate() {
        let parent = ep(Rights::READ | Rights::INVOKE);
        let err = mint(&parent, MintArgs {
            cap_type: CapType::Endpoint,
            object_id: 1,
            rights: Rights::READ,
            badge: 0,
        }).unwrap_err();
        assert_eq!(err, CapError::InsufficientRights);
    }

    #[test]
    fn mint_blocked_when_sealed() {
        let parent = Cap { sealed: true, ..ep(Rights::ALL) };
        let err = mint(&parent, MintArgs {
            cap_type: CapType::Endpoint,
            object_id: 1,
            rights: Rights::READ,
            badge: 0,
        }).unwrap_err();
        assert_eq!(err, CapError::AlreadySealed);
    }

    #[test]
    fn seal_strips_delegate_and_write() {
        let mut space: CSpace<8> = CSpace::new(0);
        space.insert(CapPtr(0), ep(Rights::ALL)).unwrap();
        seal(&mut space, CapPtr(0)).unwrap();
        let cap = space.lookup(CapPtr(0)).unwrap();
        assert!(cap.sealed);
        assert!(cap.rights.is_invoke());
        assert!(!cap.rights.can_delegate());
        assert!(!cap.rights.is_write());
    }

    #[test]
    fn double_seal_fails() {
        let mut space: CSpace<8> = CSpace::new(0);
        space.insert(CapPtr(0), ep(Rights::ALL)).unwrap();
        seal(&mut space, CapPtr(0)).unwrap();
        let err = seal(&mut space, CapPtr(0)).unwrap_err();
        assert_eq!(err, CapError::AlreadySealed);
    }

    #[test]
    fn copy_across_spaces() {
        let mut src: CSpace<8> = CSpace::new(0);
        let mut dst: CSpace<8> = CSpace::new(1);
        src.insert(CapPtr(0), ep(Rights::ALL)).unwrap();
        copy(&src, CapPtr(0), &mut dst, CapPtr(2), Rights::READ).unwrap();
        let got = dst.lookup(CapPtr(2)).unwrap();
        assert_eq!(got.rights, Rights::READ);
    }

    #[test]
    fn transfer_moves_and_clears_source() {
        let mut space: CSpace<8> = CSpace::new(0);
        space.insert(CapPtr(0), ep(Rights::ALL)).unwrap();
        transfer(&mut space, CapPtr(0), CapPtr(5)).unwrap();
        assert!(space.lookup(CapPtr(0)).unwrap().is_null());
        assert!(!space.lookup(CapPtr(5)).unwrap().is_null());
    }

    #[test]
    fn revoke_zeroes_slot() {
        let mut space: CSpace<8> = CSpace::new(0);
        space.insert(CapPtr(1), ep(Rights::ALL)).unwrap();
        revoke(&mut space, CapPtr(1)).unwrap();
        assert!(space.lookup(CapPtr(1)).unwrap().is_null());
    }
}
