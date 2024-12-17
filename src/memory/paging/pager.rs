use std::{collections::HashMap, hash::Hash, ops::{Deref, DerefMut, Index, IndexMut}, slice::SliceIndex, sync::{Arc, Weak}};

use parking_lot::Mutex;
use rand::random;

use super::{Page, PageAllocator};



/// Memory pager, this will perform swaps in and out of memory. This
/// is what should be used instead of using the allocator directly.
struct PagerInternal {
    /// This allocates pages of memory
    allocator: PageAllocator,

    /// If this is valid then it will be in the translation table.
    valid: Vec<(RawPagePtr, bool)>,


    /// Pager clock, this is what perfoms LRU
    pager_clock: u128,
    lru_map: HashMap<RawPagePtr, u128>,


    /// This translates pointers into actual page pointers.
    translation: HashMap<RawPagePtr, *const Page>,


    /// To keep things simple, we will just keep the swap space in here.
    swap: HashMap<RawPagePtr, [u8; 4096]>
}

impl PagerInternal {
    pub fn new(pages: usize) -> Self {
        let pager = Self {
            allocator: PageAllocator::new(pages),
            valid: Vec::new(),
            pager_clock: 0,
            lru_map: HashMap::new(),
            translation: HashMap::new(),
            swap: HashMap::new()
        };
        pager
    }

    /// Selects a page for swapping.
    fn select_for_swap(&mut self) -> RawPagePtr {
        // Find the least recently used page.
        let page = *self.lru_map.iter_mut().min_by_key(|(_, v)| **v).unwrap().0;
        self.lru_map.remove(&page);

        // Change the validity bit.
        let (_, valid) = self.valid.iter_mut().find(|(p, _)| *p == page).unwrap();
        *valid = false;
        
    
        page
    }
    /// Will swap a page out of memory.
    pub fn swap_out(&mut self) -> *const Page {
        // old page
        let old = self.select_for_swap();
        // get the actual pointer
        let actual = self.translation.remove(&old).unwrap();
        // store this in the swap.
        let page_data = unsafe { (*actual).data.clone() };
        self.swap.insert(old, page_data);
        // zero the old page.
        unsafe { (*actual.cast_mut()).data.fill(0); }
        actual
    }
    pub fn new_page(&mut self) -> RawPagePtr {
        let ptr = RawPagePtr(random());

        // Fill the LRU map.
        self.lru_map.insert(ptr, self.pager_clock);
        self.pager_clock += 1;

        if self.allocator.pages() != 0 {
            // We have an actual page that is ready to be
            // directly allocated.
            self.valid.push((ptr, true));
            self.translation.insert(ptr, self.allocator.acquire());
        } else {
            // We need to swap out a current frame.
            let page = self.swap_out();
            self.translation.insert(ptr, page);
            self.valid.push((ptr, true));
        }
        ptr
    }
    fn is_valid(&self, ptr: RawPagePtr) -> bool {
        self.valid.iter().find(|(a, _)| *a == ptr).unwrap().1
    }
    pub fn set_valid(&mut self, ptr: RawPagePtr, is_valid: bool) {
        let (_, valid) = self.valid.iter_mut().find(|(a, _)| *a == ptr).unwrap();
        *valid = is_valid;
    }
    pub fn refer<'b>(&mut self, ptr: RawPagePtr) -> *mut [u8; 4096] {
        
        // Update the LRU cache.
        self.lru_map.insert(ptr, self.pager_clock);
        self.pager_clock += 1;

        if self.is_valid(ptr) {
            // The reference is in memory.
            let translated = *self.translation.get(&ptr).unwrap();
            unsafe { &mut (*translated.cast_mut()).data }
        } else {
            // The reference is not in memory.
            let page = self.swap_out();
         
            // Get the swap of the old page.
            let swap = self.swap.remove(&ptr).unwrap();

            // Restore the old page contents.
            unsafe { &mut (*page.cast_mut()).data }.copy_from_slice(&swap);

            self.set_valid(ptr, true);
            self.translation.insert(ptr, page);
            unsafe { &mut (*page.cast_mut()).data }
        }

    }
}


/// This is the public API for the pager, it wraps
/// it around an [Arc] for better ergonomics, it is
/// still very usnafe.
pub struct Pager {
    internal: Arc<Mutex<PagerInternal>>
}

impl Pager {
    pub fn new(pages: usize) -> Self {
        Self {
            internal: Arc::new(Mutex::new(PagerInternal::new(pages)))
        }
    }
    pub fn alloc(&self) -> PagePtr {
        let raw = self.internal.lock().new_page();
        PagePtr(raw, Arc::downgrade(&self.internal))
    }
}



#[derive(Clone)]
pub struct PagePtr(RawPagePtr, Weak<Mutex<PagerInternal>>);

impl<Idx: SliceIndex<[u8]>> Index<Idx> for PagePtr {
    type Output = <Idx as SliceIndex<[u8]>>::Output;
    fn index(&self, index: Idx) -> &Self::Output {
        let parent = self.1.upgrade().expect("abort!");
        let pref = parent.lock().refer(self.0);
        unsafe { &*pref }.index(index)
    }
}

impl<Idx: SliceIndex<[u8]>> IndexMut<Idx> for PagePtr {
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
        let parent = self.1.upgrade().expect("abort!");
        let pref = parent.lock().refer(self.0);
        unsafe { &mut *pref }.index_mut(index)
    }
}

// impl Deref for PagePtr {
//     type Target = [u8; 4096];
//     fn deref(&self) -> &Self::Target {
//         let parent = self.1.upgrade().expect("abort!");
//         let pref = parent.lock().refer(self.0);
//         unsafe { &*pref }
//     }
// }

// impl DerefMut for PagePtr {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         let parent = self.1.upgrade().expect("abort!");
//         let pref = parent.lock().refer(self.0);
//         unsafe { &mut *pref }
//     }
// }


#[derive(Clone, Copy, Eq, Debug)]
struct RawPagePtr(usize);

impl PartialEq for RawPagePtr {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Hash for RawPagePtr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}


#[cfg(test)]
mod tests {
    use crate::memory::paging::pager::PagerInternal;

    use super::Pager;

    #[test]
    pub fn test_pager_proper() {
        // A test that uses the full API.
        let pager = Pager::new(1);

        let m1 = pager.alloc();
        let m2 = pager.alloc();

        
        
    }


    #[test]
    pub fn test_pager() {
        let mut pager = PagerInternal::new(1);
        let page = pager.new_page();
        let page2 = pager.new_page();
        let page3 = pager.new_page();
        {
            let derefed = unsafe { &mut *pager.refer(page) };
            derefed[0] = 43;
            assert_eq!(derefed[0], 43);
        }

        {
            let derefed = unsafe { &mut *pager.refer(page2) };
            assert_eq!(derefed[0], 0);
            derefed[1] = 22;
        }
        {
            let derefed = unsafe { &mut *pager.refer(page) };
            assert_eq!(derefed[0], 43);
        }
        {
            let derefed = unsafe { &mut *pager.refer(page3) };
            assert_eq!(derefed[0], 0);
            assert_eq!(derefed[1], 0);
        }
        

        
    }
}



