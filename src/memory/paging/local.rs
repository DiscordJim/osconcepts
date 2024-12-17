use rand::random;

use super::Page;


/// The logical address has the first 6 bits set
/// to the page number and then the last 10 bits
/// 
/// This is designed for little endian.
pub struct LogicalAddress(usize);

impl LogicalAddress {
    pub fn create(page: *const Page) -> (u16, Self) {
        // Generate a local header address.
        let local_root: u16 = random();

        let page_number = page as usize;

        // Extract the real root.
        let clear_mask = (!0 as usize) << 16;
        let real_root: u16 = (page_number & !clear_mask) as u16;

        // Create the modified address
        let logical = (page_number & clear_mask) | (local_root as usize);


        (real_root, Self(logical))
    }
    pub fn translate(&self, real_root: u16) -> *const Page {
        // Recreate the actual pointer address.
        let real = (self.0 & ((!0 as usize) << 16)) | (real_root as usize);
        real as *const Page
    }
}



#[cfg(test)]
mod tests {
    use rand::random;

    use crate::memory::paging::{local::LogicalAddress, PageAllocator};


    #[test]
    pub fn test_logical_address() {
        let mut allocator = PageAllocator::new(1);
        let page_ptr = allocator.acquire();

        let (root, log) = LogicalAddress::create(page_ptr);
        
        assert_eq!(log.translate(root) as usize, page_ptr as usize);
        // let page_ptr_us = page_ptr as usize;

        
        // println!("Ptr:  {:#066b}", page_ptr_us);
        // // let mask = ;
        // let real = (log.0 & ((!0 as usize) << 16)) | (root as usize);
        // assert_eq!(page_ptr_us, real);




        // let faux: u16 = random();
        // println!("Locl: {:#018b}", faux);

        // // Extract the real root.
        // let clear_mask = (!0 as usize) << 16;
        // let real: u16 = ((page_ptr_us & !(clear_mask))) as u16;

        // let log_ptr = (page_ptr_us & clear_mask) | (faux as usize);
      
        // println!("Real: {:#018b}", real);
        // println!("Lgcl: {:#066b}", log_ptr);

        // let back = (log_ptr & clear_mask) | (real as usize);
        // assert_eq!(page_ptr_us, back);
     


        // // This will extract the real first 16-bits of the address.
        // let real: usize = page_ptr_us & !(!0 >> 16);
        // let real_root: u16 = (real >> 48) as u16;


        // // Calculate the local address
        // let local: usize = (faux as usize) << 48;
        // let logical: usize = local | (page_ptr_us & (!0 >> 16));
        


        // println!("Real: {:#066b}", real);
        // println!("Faux: {:#066b}", local);
        // println!("Lgcl: {:#066b}", logical);

        
        // println!("Reas: {:#018b}", real_root);


        
        // let page = unsafe { &mut *(allocator.acquire().cast_mut()) };

        
     
        // let mut addy: u16 = 0;

        // // Get the first 6 bits.
        // let pnum_masked = page.page_number & 252;

        // addy = ((addy << 6) >> 6) | ((pnum_masked as u16) << 8);

        // println!("Page Number: {:#010b}", pnum_masked);
        // println!("Page NUmber SHifted: {:#018b}", ((pnum_masked as u16) << 8));
        // println!("Addy: {:#018b}", addy);

        panic!("test concluded.");
    }
}