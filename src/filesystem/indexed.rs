use std::{collections::HashMap, ptr};


const BLOCK_SIZE: usize = 2;

#[derive(Default)]
pub struct Directory {
    files: HashMap<String, IndexBlock>
}

impl Directory {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
    pub fn open_file(&mut self, name: String, alloc: &mut IndexedAllocator, data: &[u8]) {
        self.files.insert(name, alloc.store_file(data));
    }
    pub fn read_file(&mut self, name: &str) -> Vec<u8> {
        IndexedAllocator::read_file(self.files.get(name).unwrap())
    }
    pub fn delete_file(&mut self, name: &str, alloc: &mut IndexedAllocator) {
        let index = self.files.remove(name).unwrap();
        alloc.delete_file(&index);
    }
}

impl Block {
    pub fn collect(mut pointer: *const Block) -> Vec<*const Block> {
        let mut list = vec![];
        while !pointer.is_null() {
            list.push(pointer);
            pointer = unsafe { (*pointer).next };
        }
        list
    }
}

#[derive(Clone, Debug)]
struct Block {
    /// Points to the next block in the master list.
    next: *const Block,
    /// Bytes actually written
    length: usize,
    /// The data contained within the block.
    data: [u8; BLOCK_SIZE]
}

impl Block {
    pub fn init() -> Self {
        Self {
            next: ptr::null(),
            length: 0,
            data: [0u8; BLOCK_SIZE]
        }
    }
    pub fn alloc() -> *const Self {
        Box::leak(Box::new(Self::init())) as *const Self
    }
}


/// This is the index block that points to all
/// the file blocks.
struct IndexBlock {
    /// Link to the indexes in order.
    indexes: Vec<*const Block>
}



/// An incredibly unsafe [LinkedAllocator]
/// for use with files.
pub struct IndexedAllocator {
    /// The master list of blocks.
    blocks: *const Block,
    /// The list of free blocks.
    free_list: Vec<*const Block>,
}

impl IndexedAllocator {
    pub fn new(blocks: usize) -> Self {
        let head = Block::alloc();

        let mut free_list = vec![];
        

        // create the linked list of blocks.
        let mut current = head;
        for _ in 0..blocks - 1 {
            let next = Block::alloc();
            unsafe { (*(current as *mut Block)).next = next };
            free_list.push(current);
            current = next;
        }

        Self {
            blocks: head,
            free_list
        }
    }
    
    /// Stores a file into the linked allocator.
    fn store_file(&mut self, data: &[u8]) -> IndexBlock {
        if self.free_list.is_empty() || self.free_list.len() < data.len() / BLOCK_SIZE {
            panic!("No more room in the allocator!");
        }
        let mut offset = 0;

        
        let mut indexes = vec![];

        while offset < data.len() {
            // pop the next block.
            let block_ptr = self.free_list.pop().unwrap();
            let block = unsafe { &mut *block_ptr.cast_mut() };
            
            // push the index into the list.
            indexes.push(block_ptr);

      

            // Fill the block with data.
            let length = (offset + BLOCK_SIZE).min(data.len());
            block.data[0..length - offset].copy_from_slice(&data[offset..length]);
            block.length = length - offset;
            
            // Increase the offset
            offset += BLOCK_SIZE;
        }
        IndexBlock {
            indexes
        }
    }
    /// Delete file
    fn delete_file(&mut self, start: &IndexBlock) {
        
        for c in &start.indexes {
            let c = *c;
            let b = unsafe { &mut *c.cast_mut() };
            b.length = 0;
            self.free_list.push(c);
        }
    }
    /// Reads a file by traversing the linked list.
    fn read_file(start: &IndexBlock) -> Vec<u8> {
        let mut buffer = vec![];
        for file in &start.indexes {
            let block = unsafe { &*(*file) };
            buffer.extend_from_slice(&block.data[..block.length]);
        }

   
        buffer
    }
}


impl Drop for IndexedAllocator {
    fn drop(&mut self) {
        for tobox in Block::collect(self.blocks) {
            let boxed = unsafe { Box::from_raw(tobox as *mut Block) };
            drop(boxed);
        }

    }
}

#[cfg(test)]
mod tests {


    use crate::filesystem::indexed::Directory;

    use super::IndexedAllocator;



    #[test]
    pub fn test_indexed_allocation() {
        let mut alloc = IndexedAllocator::new(25);

        let mut directory = Directory::new();
        directory.open_file("josh".to_string(), &mut alloc, &[1,2,3]);
        directory.open_file("josh2".to_string(), &mut alloc, &[0,3,6,9]);


        assert_eq!(directory.read_file("josh"), [1,2,3]);
        assert_eq!(directory.read_file("josh2"), [0,3,6,9]);

        directory.delete_file("josh", &mut alloc);
        
    }

}