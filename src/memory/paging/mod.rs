use std::{ops::{Index, IndexMut}, slice::SliceIndex};

use rand::random;

pub mod local;

pub struct Page {
    /// This is the page number, we only use the first six bits of this.
    page_number: u8,
    /// The actual page data.
    data: [u8; 4096]
}

impl Page {
    pub fn alloc() -> *const Self {
        Box::leak(Box::new(Self {
            page_number: random(),
            data: [0u8; 4096]
        })) as *const Self
    }
    pub fn page_number(&self) -> u8 {
        self.page_number
    }
}

impl<Idx: SliceIndex<[u8]>> Index<Idx> for Page {
    type Output = <Idx as SliceIndex<[u8]>>::Output;
    fn index(&self, index: Idx) -> &Self::Output {
        &self.data[index]
    }
}

impl<Idx: SliceIndex<[u8]>> IndexMut<Idx> for Page {
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
        &mut self.data[index]
    }
}


/// The [PageAllocator] all this does is allocate raw pages of memory.
/// 
/// There are two lists, there is the page list which is our master list
/// and is used to drop the pages and then there is the free pages list
/// which keeps track of which are free.
pub struct PageAllocator {
    /// The page list.
    /// This is never modified and is used to free the memory.
    page_list: Vec<*const Page>,

    /// A list of the free pages in the page allocator.
    free_pages: Vec<*const Page>
}

impl PageAllocator {
    /// Creates a new [PageAllocator] with a certain amount of
    /// [Page]. This is very unsafe but again this is for demonstration
    /// purposes.
    pub fn new(pages: usize) -> Self {
        let mut page_list = vec![];
        let mut free_pages = vec![];
        for _ in 0..pages {
            let ptr = Page::alloc();
            page_list.push(ptr);
            free_pages.push(ptr);
        }
        Self {
            page_list,
            free_pages
        }
    }
    
    pub fn acquire(&mut self) -> *const Page {
        self.free_pages.pop().unwrap()
    }
    pub fn release(&mut self, page: *const Page) {
        // Zero the page.
        unsafe { (*page.cast_mut()).data.fill(0) };

        // Release it back to main memory.
        self.free_pages.push(page);
    }
}

impl Drop for PageAllocator {
    fn drop(&mut self) {
        for page in &self.page_list {
            let boxed_page = unsafe { Box::from_raw((*page).cast_mut()) };
            drop(boxed_page);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PageAllocator;


    #[test]
    pub fn test_page_alloc() {
        // initialize a page allocator
        let mut alloc = PageAllocator::new(1);
        let acquire = alloc.acquire();
        let array = unsafe { &mut (*acquire.cast_mut()) };

        // Now we have an array we can mess with.
        array[0] = 34;
        assert_eq!(array[0], 34);

        // let us release i.
        alloc.release(acquire);

        // acquire the page again, should be zeroed
        alloc.acquire();
        // this is still a valid reference (again this is why this is unsafe lol)
        assert_eq!(array[0], 0);

    }
}