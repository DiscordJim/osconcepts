use rand::random;

use super::Page;


/// The logical address has the first 6 bits set
/// to the page number and then the last 10 bits
pub struct LogicalAddress(usize);

impl LogicalAddress {
    pub fn create(page: *const Page) {
        // Generate a local header address.
        let local_root: u16 = random();

        // Extract the first 16-bits of the address.
        let real_root: u16 = (((page as usize) & !(!0 >> 16)) >> 48) as u16;

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

        let logical = LogicalAddress::create(page_ptr);
        let page_ptr_us = page_ptr as usize;

        
        println!("Ptr:  {:#066b}", page_ptr_us);


        let faux: u16 = random();
        println!("Locl: {:#018b}", faux);

     


        // This will extract the real first 16-bits of the address.
        let real: usize = page_ptr_us & !(!0 >> 16);
        let real_root: u16 = (real >> 48) as u16;


        // Calculate the local address
        let local: usize = (faux as usize) << 48;
        let logical: usize = local | (page_ptr_us & (!0 >> 16));
        


        println!("Real: {:#066b}", real);
        println!("Faux: {:#066b}", local);
        println!("Lgcl: {:#066b}", logical);

        
        println!("Reas: {:#018b}", real_root);


        
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