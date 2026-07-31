/// Buddy allocator over a statically-sized memory pool.
///
/// Block sizes are powers of two from 2^MIN_ORDER to 2^MAX_ORDER bytes.
/// Free blocks at each order are linked through their own memory (intrusive list).
/// No heap required — all bookkeeping lives inside the pool.

const MIN_ORDER: usize = 5;  // 32 bytes minimum block
const MAX_ORDER: usize = 14; // 16 KiB maximum block (prototype; real kernel passes in physical memory)
const NUM_ORDERS: usize = MAX_ORDER - MIN_ORDER + 1;
const POOL_SIZE: usize = 1 << MAX_ORDER; // 16 KiB pool

const NULL_OFFSET: u32 = u32::MAX;

/// Intrusive linked-list node embedded at the start of each free block.
#[repr(C)]
struct FreeBlock {
    next: u32, // byte offset from pool base, NULL_OFFSET = end of list
}

pub struct BuddyAllocator {
    pool:       [u8; POOL_SIZE],
    free_lists: [u32; NUM_ORDERS], // head offset per order, NULL_OFFSET = empty
    allocated:  usize,
}

impl BuddyAllocator {
    pub fn new() -> Self {
        let mut this = Self {
            pool:       [0u8; POOL_SIZE],
            free_lists: [NULL_OFFSET; NUM_ORDERS],
            allocated:  0,
        };
        // push_free writes the intrusive next=NULL_OFFSET sentinel into the pool bytes
        // so that pop_free reads the correct terminator on the first allocation.
        this.push_free(NUM_ORDERS - 1, 0);
        this
    }

    fn order_for_size(size: usize) -> Option<usize> {
        if size == 0 {
            return None;
        }
        let min_block = 1usize << MIN_ORDER;
        let size = size.max(min_block);
        let bits = (size - 1).leading_zeros() as usize;
        let order_abs = usize::BITS as usize - bits; // ceil log2
        if order_abs < MIN_ORDER || order_abs > MAX_ORDER {
            return None;
        }
        Some(order_abs - MIN_ORDER)
    }

    fn block_size(order: usize) -> usize {
        1 << (order + MIN_ORDER)
    }

    fn buddy_offset(offset: u32, order: usize) -> u32 {
        offset ^ (Self::block_size(order) as u32)
    }

    unsafe fn write_next(&mut self, offset: usize, next: u32) {
        let node = &mut *(self.pool.as_mut_ptr().add(offset) as *mut FreeBlock);
        node.next = next;
    }

    unsafe fn read_next(&self, offset: usize) -> u32 {
        let node = &*(self.pool.as_ptr().add(offset) as *const FreeBlock);
        node.next
    }

    fn push_free(&mut self, order: usize, offset: u32) {
        let old_head = self.free_lists[order];
        unsafe { self.write_next(offset as usize, old_head) };
        self.free_lists[order] = offset;
    }

    fn pop_free(&mut self, order: usize) -> Option<u32> {
        let head = self.free_lists[order];
        if head == NULL_OFFSET {
            return None;
        }
        let next = unsafe { self.read_next(head as usize) };
        self.free_lists[order] = next;
        Some(head)
    }

    fn remove_from_free_list(&mut self, order: usize, target: u32) -> bool {
        let mut prev = NULL_OFFSET;
        let mut cur = self.free_lists[order];
        while cur != NULL_OFFSET {
            let next = unsafe { self.read_next(cur as usize) };
            if cur == target {
                if prev == NULL_OFFSET {
                    self.free_lists[order] = next;
                } else {
                    unsafe { self.write_next(prev as usize, next) };
                }
                return true;
            }
            prev = cur;
            cur = next;
        }
        false
    }

    /// Allocate `size` bytes, returning a byte offset into the pool.
    pub fn alloc(&mut self, size: usize) -> Option<u32> {
        let target_order = Self::order_for_size(size)?;

        // Find the smallest order >= target_order that has a free block
        let mut split_order = (target_order..NUM_ORDERS)
            .find(|&o| self.free_lists[o] != NULL_OFFSET)?;

        let offset = self.pop_free(split_order).unwrap();

        // Split down to target_order, pushing buddies to free lists
        while split_order > target_order {
            split_order -= 1;
            let buddy = offset + Self::block_size(split_order) as u32;
            self.push_free(split_order, buddy);
        }

        // Zero the allocated block
        let bs = Self::block_size(target_order);
        let slice = &mut self.pool[offset as usize..offset as usize + bs];
        for b in slice.iter_mut() { *b = 0; }

        self.allocated += bs;
        Some(offset)
    }

    /// Free a previously allocated block at `offset` of `size` bytes.
    pub fn free(&mut self, offset: u32, size: usize) {
        let Some(mut order) = Self::order_for_size(size) else { return };
        let bs = Self::block_size(order);
        self.allocated = self.allocated.saturating_sub(bs);

        let mut cur = offset;
        // Merge with buddy while the buddy is free and we're below MAX_ORDER
        while order < NUM_ORDERS - 1 {
            let buddy = Self::buddy_offset(cur, order);
            if buddy as usize + Self::block_size(order) > POOL_SIZE {
                break;
            }
            if self.remove_from_free_list(order, buddy) {
                cur = cur.min(buddy); // merged block starts at the lower address
                order += 1;
            } else {
                break;
            }
        }
        self.push_free(order, cur);
    }

    pub fn allocated_bytes(&self) -> usize { self.allocated }
    pub fn pool_size() -> usize { POOL_SIZE }

    /// Get a pointer into the pool for an allocated offset (unsafe: caller must ensure validity).
    pub unsafe fn ptr(&mut self, offset: u32) -> *mut u8 {
        self.pool.as_mut_ptr().add(offset as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_and_free_small() {
        let mut ba = BuddyAllocator::new();
        let off = ba.alloc(64).expect("alloc 64");
        assert_eq!(off % 64, 0, "must be 64-aligned");
        assert_eq!(ba.allocated_bytes(), 64);
        ba.free(off, 64);
        assert_eq!(ba.allocated_bytes(), 0);
    }

    #[test]
    fn alloc_many_disjoint() {
        let mut ba = BuddyAllocator::new();
        let a = ba.alloc(32).unwrap();
        let b = ba.alloc(32).unwrap();
        assert_ne!(a, b);
        // Ranges must not overlap
        assert!(a + 32 <= b || b + 32 <= a);
    }

    #[test]
    fn zero_on_alloc() {
        let mut ba = BuddyAllocator::new();
        let off = ba.alloc(32).unwrap();
        let slice = &ba.pool[off as usize..off as usize + 32];
        assert!(slice.iter().all(|&b| b == 0));
    }

    #[test]
    fn coalesce_after_free() {
        let mut ba = BuddyAllocator::new();
        let a = ba.alloc(32).unwrap();
        let b = ba.alloc(32).unwrap();
        ba.free(a, 32);
        ba.free(b, 32);
        // After merging, a large block should be allocatable
        let big = ba.alloc(POOL_SIZE / 2).expect("should coalesce back");
        assert!(big < POOL_SIZE as u32);
    }

    #[test]
    fn alloc_max_order() {
        let mut ba = BuddyAllocator::new();
        let off = ba.alloc(POOL_SIZE).expect("alloc entire pool");
        assert_eq!(off, 0);
        assert!(ba.alloc(32).is_none(), "pool exhausted");
        ba.free(off, POOL_SIZE);
        assert!(ba.alloc(32).is_some(), "pool recovered");
    }
}
