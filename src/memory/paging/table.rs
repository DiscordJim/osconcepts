use std::{collections::HashMap, sync::Arc};

use super::{local::LogicalAddress, pager::{PagePtr, Pager}};


/// This [PageTable] will translate local addresses into
/// actual addresses that can be derefered into a pager.
pub struct PageTable {
    /// The [super::Page] translation table.
    mapping: HashMap<u16, u16>,

    /// Pager,
    pager: Arc<Pager>
}

impl PageTable {
    pub fn new(pager: Arc<Pager>) -> Self {
        Self {
            mapping: HashMap::default(),
            pager
        }
    }
    /// Allocates a page to the local process and will return a [LogicalAddress]
    /// in a real machine this would be a system call.
    pub fn alloc(&mut self) -> LogicalAddress {
        let (real, logical) = LogicalAddress::create(self.pager.alloc());
        self.mapping.insert(logical.logical_root(), real);
        logical
    }
    /// Performs a page reference. Needless to say this is incredibly unsafe.
    pub fn reference(&self, ptr: LogicalAddress) -> PagePtr {
        let real = *self.mapping.get(&ptr.logical_root()).unwrap();
        ptr.translate(real, self.pager.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::memory::paging::pager::Pager;

    use super::PageTable;



    #[test]
    pub fn test_page_table() {
        let page_alloc = Arc::new(Pager::new(18));
        let mut page_table = PageTable::new(page_alloc);
        
        // Gets the local address
        let local = page_table.alloc();
        let mut page_frame = page_table.reference(local);

        page_frame[3] = 4;
        assert_eq!(page_frame[3], 4);
    }

}