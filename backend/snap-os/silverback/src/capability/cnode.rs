use super::rights::Rights;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CapPtr(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum CapType {
    Null         = 0,
    Untyped      = 1,
    CNode        = 2,
    Tcb          = 3,
    Endpoint     = 4,
    Notification = 5,
    Frame        = 6,
    PageTable    = 7,
    AsidPool     = 8,
    IrqControl   = 9,
    IrqHandler   = 10,
    SoulCap      = 11,
    BifrostCap   = 12,
    DeviceCap    = 13,
}

#[derive(Clone, Copy, Debug)]
pub struct Cap {
    pub cap_type:  CapType,
    pub object_id: u64,
    pub rights:    Rights,
    pub badge:     u64,
    pub sealed:    bool,
}

impl Cap {
    pub const NULL: Self = Self {
        cap_type:  CapType::Null,
        object_id: 0,
        rights:    Rights::NONE,
        badge:     0,
        sealed:    false,
    };

    pub const fn is_null(&self) -> bool {
        matches!(self.cap_type, CapType::Null)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapError {
    InvalidSlot,
    SlotOccupied,
    NullCap,
    InsufficientRights,
    AlreadySealed,
}

/// Fixed-size capability node — N slots, no heap.
pub struct CNode<const N: usize> {
    slots: [Cap; N],
}

impl<const N: usize> CNode<N> {
    pub const fn new() -> Self {
        Self { slots: [Cap::NULL; N] }
    }

    pub fn get(&self, ptr: CapPtr) -> Result<&Cap, CapError> {
        self.slots.get(ptr.0).ok_or(CapError::InvalidSlot)
    }

    fn get_mut(&mut self, ptr: CapPtr) -> Result<&mut Cap, CapError> {
        self.slots.get_mut(ptr.0).ok_or(CapError::InvalidSlot)
    }

    pub fn insert(&mut self, ptr: CapPtr, cap: Cap) -> Result<(), CapError> {
        let slot = self.get_mut(ptr)?;
        if !slot.is_null() {
            return Err(CapError::SlotOccupied);
        }
        *slot = cap;
        Ok(())
    }

    pub fn delete(&mut self, ptr: CapPtr) -> Result<Cap, CapError> {
        let slot = self.get_mut(ptr)?;
        if slot.is_null() {
            return Err(CapError::NullCap);
        }
        let cap = *slot;
        *slot = Cap::NULL;
        Ok(cap)
    }

    pub fn size(&self) -> usize { N }
}

/// Per-soul capability namespace backed by a root CNode.
pub struct CSpace<const N: usize> {
    pub soul_id: u64,
    root: CNode<N>,
}

impl<const N: usize> CSpace<N> {
    pub const fn new(soul_id: u64) -> Self {
        Self { soul_id, root: CNode::new() }
    }

    pub fn lookup(&self, ptr: CapPtr) -> Result<&Cap, CapError> {
        self.root.get(ptr)
    }

    pub fn insert(&mut self, ptr: CapPtr, cap: Cap) -> Result<(), CapError> {
        self.root.insert(ptr, cap)
    }

    pub fn delete(&mut self, ptr: CapPtr) -> Result<Cap, CapError> {
        self.root.delete(ptr)
    }

    pub fn cnode_size(&self) -> usize { N }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cap(id: u64) -> Cap {
        Cap {
            cap_type:  CapType::Endpoint,
            object_id: id,
            rights:    Rights::ALL,
            badge:     0,
            sealed:    false,
        }
    }

    #[test]
    fn insert_and_lookup() {
        let mut cs: CSpace<16> = CSpace::new(1);
        let cap = make_cap(42);
        cs.insert(CapPtr(0), cap).unwrap();
        let found = cs.lookup(CapPtr(0)).unwrap();
        assert_eq!(found.object_id, 42);
    }

    #[test]
    fn double_insert_fails() {
        let mut cs: CSpace<16> = CSpace::new(1);
        cs.insert(CapPtr(0), make_cap(1)).unwrap();
        let err = cs.insert(CapPtr(0), make_cap(2)).unwrap_err();
        assert_eq!(err, CapError::SlotOccupied);
    }

    #[test]
    fn delete_clears_slot() {
        let mut cs: CSpace<16> = CSpace::new(1);
        cs.insert(CapPtr(3), make_cap(99)).unwrap();
        let removed = cs.delete(CapPtr(3)).unwrap();
        assert_eq!(removed.object_id, 99);
        let err = cs.delete(CapPtr(3)).unwrap_err();
        assert_eq!(err, CapError::NullCap);
    }

    #[test]
    fn out_of_bounds_slot() {
        let cs: CSpace<4> = CSpace::new(1);
        let err = cs.lookup(CapPtr(4)).unwrap_err();
        assert_eq!(err, CapError::InvalidSlot);
    }

    #[test]
    fn null_cap_is_null() {
        assert!(Cap::NULL.is_null());
    }
}
