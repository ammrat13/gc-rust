use std::alloc::Layout;

pub struct GCAllocation {
    pub layout: Layout,
    pub marked: bool,
    pub start: usize,
}

impl GCAllocation {
    pub fn end(&self) -> usize {
        self.start + self.layout.size()
    }
    pub fn contains(&self, ptr: usize) -> bool {
        ptr >= self.start && ptr < self.start + self.layout.size()
    }
}
