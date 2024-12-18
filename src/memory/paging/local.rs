use rand::random;

use super::pager::{PagePtr, Pager};


/// The logical address has the first 6 bits set
/// to the page number and then the last 10 bits
/// 
/// This is designed for little endian.
#[derive(Clone, Copy, PartialEq)]
pub struct LogicalAddress(usize);

impl LogicalAddress {
    /// Creates a new logical address from a page pointer, it will
    /// return the real root (page number) of the address and the 
    /// logical address.
    pub fn create(page: PagePtr) -> (u16, Self) {
        // Generate a local header address.
        let local_root: u16 = random();

        let page_number = page.addr() as usize;

        // Extract the real root.
        let clear_mask = (!0 as usize) << 16;
        let real_root: u16 = (page_number & !clear_mask) as u16;

        // Create the modified address
        let logical = (page_number & clear_mask) | (local_root as usize);


        (real_root, Self(logical))
    }
    /// Extracts the logical root.
    pub fn logical_root(&self) -> u16 {
        let clear_mask = (!0 as usize) << 16;
        (self.0 & !clear_mask) as u16
    }
    /// Translate the local address ino an actual address with the real rooot.
    pub fn translate(&self, real_root: u16, pager: &Pager) -> PagePtr {
        // Recreate the actual pointer address.
        let real = (self.0 & ((!0 as usize) << 16)) | (real_root as usize);
        unsafe { PagePtr::from_raw(real, pager) }
    }
}



#[cfg(test)]
mod tests {
    use crate::memory::paging::{local::LogicalAddress, pager::Pager};


    #[test]
    pub fn test_logical_address() {
        let allocator = Pager::new(1);
        let page_ptr = allocator.alloc();

        let (root, log) = LogicalAddress::create(page_ptr.clone());
        
        assert_eq!(page_ptr, log.translate(root, &allocator));
    }
}