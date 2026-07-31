#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct Rights(pub u8);

impl Rights {
    pub const NONE:     Self = Self(0b00000);
    pub const READ:     Self = Self(0b00001);
    pub const WRITE:    Self = Self(0b00010);
    pub const INVOKE:   Self = Self(0b00100);
    pub const DELEGATE: Self = Self(0b01000);
    pub const SEAL:     Self = Self(0b10000);
    pub const ALL:      Self = Self(0b11111);

    pub const fn has(self, right: Self) -> bool {
        (self.0 & right.0) == right.0
    }

    pub const fn restrict(self, mask: Self) -> Self {
        Self(self.0 & mask.0)
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn is_read(self) -> bool     { self.has(Self::READ) }
    pub const fn is_write(self) -> bool    { self.has(Self::WRITE) }
    pub const fn is_invoke(self) -> bool   { self.has(Self::INVOKE) }
    pub const fn can_delegate(self) -> bool { self.has(Self::DELEGATE) }
    pub const fn can_seal(self) -> bool    { self.has(Self::SEAL) }
}

impl core::ops::BitOr for Rights {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self { self.union(rhs) }
}

impl core::ops::BitAnd for Rights {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self { self.restrict(rhs) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restrict_never_amplifies() {
        let restricted = Rights::READ;
        let attempted = restricted.restrict(Rights::ALL);
        assert_eq!(attempted, Rights::READ);
    }

    #[test]
    fn delegate_check() {
        assert!(Rights::ALL.can_delegate());
        assert!(!Rights::READ.can_delegate());
    }

    #[test]
    fn bitwise_ops() {
        let rw = Rights::READ | Rights::WRITE;
        assert!(rw.is_read());
        assert!(rw.is_write());
        assert!(!rw.is_invoke());
        let r_only = rw & Rights::READ;
        assert!(r_only.is_read());
        assert!(!r_only.is_write());
    }
}
