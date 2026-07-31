/// Immix garbage collector — mark-region, non-moving prototype.
///
/// Heap is organized as:
///   Block (32 KiB) = 256 × Line (128 bytes)
/// Allocation is bump-pointer within the current line.
/// On GC, lines containing live objects are marked; unmarked lines are reclaimed.
/// Blocks with some live lines are "recyclable"; blocks with no live lines are "free".
///
/// Object layout (header at ptr, payload at ptr + HEADER_SIZE):
///   [ type_tag: u32 | size: u32 | mark: u8 | _pad: [u8;7] ]  16 bytes
///   [ payload …                                             ]
///
/// This is non-moving (no evacuation yet). GcPtr is a raw *mut u8 to the header.

pub const LINE_SIZE:    usize = 128;
pub const LINES_PER_BLOCK: usize = 256;
pub const BLOCK_SIZE:   usize = LINE_SIZE * LINES_PER_BLOCK; // 32 KiB
pub const HEADER_SIZE:  usize = 16;

#[repr(C)]
struct ObjHeader {
    type_tag: u32,
    size:     u32, // payload size in bytes
    mark:     u8,
    _pad:     [u8; 7],
}

const _: () = assert!(core::mem::size_of::<ObjHeader>() == HEADER_SIZE);

/// A 32 KiB heap block.
struct Block {
    data:     Box<[u8; BLOCK_SIZE]>,
    line_map: [u8; LINES_PER_BLOCK], // 0=free, 1=marked
    bump:     usize,                  // byte offset of next free byte in this block
}

impl Block {
    fn new() -> Self {
        Self {
            data:     Box::new([0u8; BLOCK_SIZE]),
            line_map: [0u8; LINES_PER_BLOCK],
            bump:     0,
        }
    }

    fn base(&self) -> *const u8 { self.data.as_ptr() }
    fn base_mut(&mut self) -> *mut u8 { self.data.as_mut_ptr() }

    fn line_of(&self, byte_offset: usize) -> usize { byte_offset / LINE_SIZE }

    /// Try to allocate `total` bytes (header + payload). Returns byte offset or None.
    fn try_alloc(&mut self, total: usize) -> Option<usize> {
        let aligned = (self.bump + 7) & !7; // 8-byte align
        if aligned + total > BLOCK_SIZE {
            return None;
        }
        self.bump = aligned + total;
        Some(aligned)
    }

    /// Count free (unmarked) lines.
    fn free_lines(&self) -> usize {
        self.line_map.iter().filter(|&&m| m == 0).count()
    }

    /// True if every allocated line is marked (nothing to reclaim).
    fn is_full(&self) -> bool {
        let used_lines = (self.bump + LINE_SIZE - 1) / LINE_SIZE;
        self.line_map[..used_lines].iter().all(|&m| m == 1)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockState {
    Free,
    Recyclable,
    Full,
}

/// An opaque pointer into the GC heap (points to the object header).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GcPtr(pub *mut u8);

unsafe impl Send for GcPtr {}

impl GcPtr {
    pub const NULL: Self = GcPtr(core::ptr::null_mut());

    pub fn is_null(self) -> bool { self.0.is_null() }

    /// Pointer to the payload (past the header).
    pub fn payload(self) -> *mut u8 {
        unsafe { self.0.add(HEADER_SIZE) }
    }

    unsafe fn header(self) -> *mut ObjHeader {
        self.0 as *mut ObjHeader
    }

    pub fn type_tag(self) -> u32 {
        unsafe { (*self.header()).type_tag }
    }

    pub fn payload_size(self) -> u32 {
        unsafe { (*self.header()).size }
    }

    fn mark(self) {
        unsafe { (*self.header()).mark = 1 }
    }

    fn is_marked(self) -> bool {
        unsafe { (*self.header()).mark == 1 }
    }

    fn clear_mark(self) {
        unsafe { (*self.header()).mark = 0 }
    }
}

/// The Immix heap: a list of blocks + the root set.
pub struct ImmixHeap {
    blocks:  Vec<Block>,
    current: usize, // index of the block we're currently allocating from
    roots:   Vec<GcPtr>,
}

impl ImmixHeap {
    pub fn new() -> Self {
        let mut h = Self { blocks: vec![], current: 0, roots: vec![] };
        h.blocks.push(Block::new());
        h
    }

    /// Register a root pointer. Roots are traced during GC.
    pub fn add_root(&mut self, ptr: GcPtr) {
        if !ptr.is_null() {
            self.roots.push(ptr);
        }
    }

    /// Remove a root.
    pub fn remove_root(&mut self, ptr: GcPtr) {
        self.roots.retain(|r| r.0 != ptr.0);
    }

    /// Allocate `payload_size` bytes. Returns a GcPtr to the header.
    /// Triggers a GC cycle if no block has room and allocation still fails.
    pub fn alloc(&mut self, payload_size: u32, type_tag: u32) -> Option<GcPtr> {
        let total = HEADER_SIZE + payload_size as usize;
        if total > BLOCK_SIZE {
            return None; // object too large for a single block
        }

        // Fast path: allocate from current block.
        if let Some(off) = self.blocks[self.current].try_alloc(total) {
            return Some(self.init_header(self.current, off, payload_size, type_tag));
        }

        // Try an existing free block.
        if let Some(idx) = self.find_free_block() {
            self.current = idx;
            if let Some(off) = self.blocks[self.current].try_alloc(total) {
                return Some(self.init_header(self.current, off, payload_size, type_tag));
            }
        }

        // GC cycle.
        self.collect();

        // Retry after GC.
        if let Some(idx) = self.find_free_block() {
            self.current = idx;
            if let Some(off) = self.blocks[self.current].try_alloc(total) {
                return Some(self.init_header(self.current, off, payload_size, type_tag));
            }
        }

        // Grow heap.
        self.blocks.push(Block::new());
        self.current = self.blocks.len() - 1;
        let off = self.blocks[self.current].try_alloc(total)?;
        Some(self.init_header(self.current, off, payload_size, type_tag))
    }

    fn init_header(&mut self, block_idx: usize, offset: usize, size: u32, type_tag: u32) -> GcPtr {
        let ptr = unsafe { self.blocks[block_idx].base_mut().add(offset) };
        unsafe {
            let h = ptr as *mut ObjHeader;
            (*h).type_tag = type_tag;
            (*h).size     = size;
            (*h).mark     = 0;
            (*h)._pad     = [0u8; 7];
        }
        GcPtr(ptr)
    }

    fn find_free_block(&self) -> Option<usize> {
        self.blocks.iter().enumerate().find(|(_, b)| b.free_lines() > 4).map(|(i, _)| i)
    }

    // ── GC: mark-sweep ───────────────────────────────────────────────────────

    pub fn collect(&mut self) {
        self.mark_phase();
        self.sweep_phase();
    }

    fn mark_phase(&mut self) {
        // Mark all objects reachable from roots.
        // Non-moving: just set the mark bit and the containing lines.
        let roots: Vec<GcPtr> = self.roots.clone();
        for root in roots {
            self.mark_object(root);
        }
    }

    fn mark_object(&mut self, ptr: GcPtr) {
        if ptr.is_null() || ptr.is_marked() {
            return;
        }
        ptr.mark();

        // Mark lines this object occupies.
        let (block_idx, byte_off) = self.ptr_location(ptr);
        if let Some(idx) = block_idx {
            let block = &mut self.blocks[idx];
            let start_line = block.line_of(byte_off);
            let total = HEADER_SIZE + ptr.payload_size() as usize;
            let end_line = block.line_of(byte_off + total.saturating_sub(1));
            for l in start_line..=end_line.min(LINES_PER_BLOCK - 1) {
                block.line_map[l] = 1;
            }
        }
        // Shallow mark only — payload field scanning is the caller's responsibility
        // (in a full GC, the type_tag would dispatch to a field-tracing function).
    }

    fn ptr_location(&self, ptr: GcPtr) -> (Option<usize>, usize) {
        let addr = ptr.0 as usize;
        for (i, block) in self.blocks.iter().enumerate() {
            let base = block.base() as usize;
            if addr >= base && addr < base + BLOCK_SIZE {
                return (Some(i), addr - base);
            }
        }
        (None, 0)
    }

    fn sweep_phase(&mut self) {
        for block in &mut self.blocks {
            // Reset line map for lines with no marked objects.
            // Walk the bump range looking for unmarked headers.
            let mut offset = 0;
            while offset + HEADER_SIZE <= block.bump {
                let aligned = (offset + 7) & !7;
                if aligned + HEADER_SIZE > block.bump {
                    break;
                }
                let ptr = GcPtr(unsafe { block.base_mut().add(aligned) });
                let total = HEADER_SIZE + ptr.payload_size() as usize;

                if ptr.is_marked() {
                    ptr.clear_mark();
                } else {
                    // Reclaim: zero out so future alloc gets clean memory
                    unsafe {
                        core::ptr::write_bytes(ptr.0, 0, total);
                    }
                    // Clear lines this object occupies
                    let start_line = block.line_of(aligned);
                    let end_line = block.line_of((aligned + total).saturating_sub(1));
                    for l in start_line..=end_line.min(LINES_PER_BLOCK - 1) {
                        block.line_map[l] = 0;
                    }
                }
                offset = aligned + total;
            }
        }

        // Reset bump pointer in fully-free blocks so they can be reused.
        for block in &mut self.blocks {
            if block.line_map.iter().all(|&m| m == 0) {
                block.bump = 0;
            }
        }
    }

    pub fn block_count(&self) -> usize { self.blocks.len() }

    pub fn block_state(&self, idx: usize) -> BlockState {
        let b = &self.blocks[idx];
        if b.line_map.iter().all(|&m| m == 0) {
            BlockState::Free
        } else if b.is_full() {
            BlockState::Full
        } else {
            BlockState::Recyclable
        }
    }

    pub fn heap_bytes_used(&self) -> usize {
        self.blocks.iter().map(|b| b.bump).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_and_read_back() {
        let mut heap = ImmixHeap::new();
        let ptr = heap.alloc(64, 42).expect("alloc 64 bytes");
        assert!(!ptr.is_null());
        assert_eq!(ptr.type_tag(), 42);
        assert_eq!(ptr.payload_size(), 64);
    }

    #[test]
    fn gc_reclaims_unreachable() {
        let mut heap = ImmixHeap::new();
        let _ptr = heap.alloc(32, 1).unwrap();
        // _ptr is NOT added as a root → should be reclaimed
        heap.collect();
        // After GC, the block's line map should show lines freed
        assert_eq!(heap.block_state(0), BlockState::Free);
    }

    #[test]
    fn gc_retains_roots() {
        let mut heap = ImmixHeap::new();
        let ptr = heap.alloc(32, 7).unwrap();
        heap.add_root(ptr);
        heap.collect();
        // Root must survive: its type_tag should still be readable
        assert_eq!(ptr.type_tag(), 7);
    }

    #[test]
    fn alloc_many_triggers_grow() {
        let mut heap = ImmixHeap::new();
        // Each object = 80 bytes (16 header + 64 payload, 8-aligned).
        // One block = 32 KiB → ~409 objects. Alloc one block's worth plus 10,
        // pinning each as a root so GC cannot reclaim them and must grow.
        let objects_per_block = BLOCK_SIZE / (HEADER_SIZE + 64);
        let target = objects_per_block + 10;
        let mut count = 0;
        for _ in 0..target {
            if let Some(ptr) = heap.alloc(64, 0) {
                heap.add_root(ptr); // pin → GC cannot reclaim → heap must grow
                count += 1;
            }
        }
        assert!(count > 0);
        assert!(heap.block_count() >= 2, "heap should have grown to at least 2 blocks");
    }

    #[test]
    fn payload_pointer_is_past_header() {
        let mut heap = ImmixHeap::new();
        let ptr = heap.alloc(8, 99).unwrap();
        let diff = ptr.payload() as usize - ptr.0 as usize;
        assert_eq!(diff, HEADER_SIZE);
    }
}
