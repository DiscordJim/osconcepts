use std::collections::HashMap;

use super::{local::LogicalAddress, Page, PageAllocator};


#[derive(Default)]
pub struct PageTable {
    /// The [super::Page] translation table.
    mapping: HashMap<u16, u16>,
    /// Checks if a page is in memory,
    valid: HashMap<u16, bool>
}

impl PageTable {
    /// Allocates a page to the local process and will return a [LogicalAddress]
    /// in a real machine this would be a system call.
    pub fn alloc(&mut self, allocator: &mut PageAllocator) -> LogicalAddress {
        let (real, logical) = LogicalAddress::create(allocator.acquire());
        self.mapping.insert(logical.logical_root(), real);
        self.valid.insert(logical.logical_root(), true);
        logical
    }
    /// Performs a page reference. Needless to say this is incredibly unsafe.
    pub fn reference(&self, ptr: LogicalAddress) -> *const Page {
        let real = *self.mapping.get(&ptr.logical_root()).unwrap();
        ptr.translate(real)
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::paging::PageAllocator;

    use super::PageTable;



    #[test]
    pub fn test_page_table() {
        let mut page_alloc = PageAllocator::new(18);
        let mut page_table = PageTable::default();
        
        // Gets the local address
        let local = page_table.alloc(&mut page_alloc);

        let page_frame = unsafe { &mut *page_table.reference(local).cast_mut() };
        
        page_frame[3] = 4;
        assert_eq!(page_frame[3], 4);

    }

}