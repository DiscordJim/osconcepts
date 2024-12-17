use std::{collections::HashMap, ptr};


const BLOCK_SIZE: usize = 2;


pub struct Directory {
    files: HashMap<String, *const Block>
}

impl Directory {
    pub fn new() -> Self {
        Self {
            files: HashMap::new()
        }
    }
    pub fn open_file(&mut self, name: String, alloc: &mut LinkedAllocator, data: &[u8]) {
        self.files.insert(name, alloc.store_file(data));
    }
    pub fn read_file(&mut self, name: &str) -> Vec<u8> {
        LinkedAllocator::read_file(*self.files.get(name).unwrap())
    }
    pub fn delete_file(&mut self, name: &str, alloc: &mut LinkedAllocator) {
        let index = self.files.remove(name).unwrap();
        alloc.delete_file(index);
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
pub struct Block {
    /// Points to the next block in the master list.
    next: *const Block,
    /// Points to the next block in a file.
    file_pointer: *const Block,
    /// Bytes actually written
    length: usize,
    /// The data contained within the block.
    data: [u8; BLOCK_SIZE]
}


/// An incredibly unsafe [LinkedAllocator]
/// for use with files.
pub struct LinkedAllocator {
    /// The master list of blocks.
    blocks: *const Block,
    /// The list of free blocks.
    free_list: Vec<*const Block>,
}

impl LinkedAllocator {
    pub fn new(blocks: usize) -> Self {
        let head = Box::leak(Box::new(Block {
            next: ptr::null(),
            file_pointer: ptr::null(),
            length: 0,
            data: [0u8; BLOCK_SIZE]
        })) as *const Block;

        let mut free_list = vec![];
        

        // create the linked list of blocks.
        let mut current = head;
        for _ in 0..blocks - 1 {
            let next = Box::leak(Box::new(Block {
                next: ptr::null(),
                file_pointer: ptr::null(),
                length: 0,
                data: [0u8; BLOCK_SIZE]
            }));
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
    pub fn store_file(&mut self, data: &[u8]) -> *const Block {
        if self.free_list.is_empty() || self.free_list.len() < data.len() / BLOCK_SIZE {
            panic!("No more room in the allocator!");
        }
        let mut offset = 0;
        let mut ptr: *const Block = ptr::null();
        let mut prev: *const Block = ptr::null();
        
        while offset < data.len() {
            // pop the next block.
            let block_ptr = self.free_list.pop().unwrap();
            let block = unsafe { &mut *block_ptr.cast_mut() };
            if ptr.is_null() {
                ptr = block_ptr;
                prev = block_ptr;
            } else {
                // Configure the file pointers.
                unsafe { (*prev.cast_mut()).file_pointer = block_ptr };
                prev = block_ptr;
            }

      

            // Fill the block with data.
            let length = (offset + BLOCK_SIZE).min(data.len());
            block.data[0..length - offset].copy_from_slice(&data[offset..length]);
            block.length = length - offset;
            
            // Increase the offset
            offset += BLOCK_SIZE;
        }
        ptr
    }
    /// Delete file
    pub fn delete_file(&mut self, start: *const Block) {
        let collected = Block::collect(start);
        for c in collected {
            let b = unsafe { &mut *c.cast_mut() };
            b.length = 0;
            b.file_pointer = ptr::null();
            self.free_list.push(c);
        }
    }
    /// Reads a file by traversing the linked list.
    pub fn read_file(start: *const Block) -> Vec<u8> {
        let mut current = start;
        let mut buffer = vec![];
        while !current.is_null() {
            let cur = unsafe { &*current };
            buffer.extend_from_slice(&cur.data[..cur.length]);
            current = cur.file_pointer;
        }

        buffer
    }
}


impl Drop for LinkedAllocator {
    fn drop(&mut self) {
        for tobox in Block::collect(self.blocks) {
            let boxed = unsafe { Box::from_raw(tobox as *mut Block) };
            drop(boxed);
        }

    }
}

#[cfg(test)]
mod tests {
    use crate::filesystem::linked::Directory;

    use super::LinkedAllocator;



    #[test]
    pub fn test_linked_allocation() {
        let mut alloc = LinkedAllocator::new(25);

        let mut directory = Directory::new();
        directory.open_file("josh".to_string(), &mut alloc, &[1,2,3]);
        directory.open_file("josh2".to_string(), &mut alloc, &[0,3,6,9]);


        assert_eq!(directory.read_file("josh"), [1,2,3]);
        assert_eq!(directory.read_file("josh2"), [0,3,6,9]);
        
    }

    
}