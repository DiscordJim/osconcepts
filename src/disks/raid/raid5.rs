use std::sync::atomic::{AtomicUsize, Ordering};

use crate::disks::{bits::BitVec, hard_drive::MagneticDisk, AbstractStorageDevice, RawStoragePtr};


#[derive(Default)]
pub struct Raid5Builder {
    array: Vec<MagneticDisk>,
    block_size: Option<usize>
}

impl Raid5Builder {
    pub fn with_disk(mut self, disk: MagneticDisk) -> Self {
        self.array.push(disk);
        self
    }
    pub fn with_block_size(mut self, block: usize) -> Self {
        self.block_size = Some(block);
        self
    }
    pub fn build(self) -> Raid5 {
        Raid5 {
            array: self.array.into_boxed_slice(),
            block_size: self.block_size.unwrap(),
            offset: AtomicUsize::new(0)
        }
    }
}

/// A RAID5 array. 
/// 
/// Byte-level (it should be block but this is a huge pain so we are doing it at
/// the byte level) striping w/ no dedicated disk.
pub struct Raid5 {
    array: Box<[MagneticDisk]>,
    block_size: usize,
    offset: AtomicUsize
}

impl Raid5 {
    /// Writes to the RAID4 array, performing striping at the
    /// byte level.
    /// 
    /// This will only perform byte level writes.
    pub fn write(&self, data: &[u8]) -> RawStoragePtr {
        let ptr = RawStoragePtr::byte_ptr(self.offset.load(Ordering::SeqCst));
        let current_offset = self.offset.load(Ordering::SeqCst);
        

        for i in 0..data.len() {
            // Get the index of the block.
            let block = (i + current_offset) / self.block_size;

            // Get the offset of the block.
            let disk_index = block % self.array.len();

            // This calculates the offset from he block, so the acual byte position.
            let inner = block + ((i - (block * self.block_size)) % self.block_size);
            
            self.array[disk_index].write(RawStoragePtr::byte_ptr(inner), &[data[i]]).get();
            // disk.write(RawStoragePtr::byte_ptr(byte_offset), &data[])

            // for byte_index in 0..self.block_size {
            //     let full_index = (block * self.block_size) + byte_index;
            //     println!("Full Index: {}", full_index);
            // }

            // println!("Block: {}, Block Offset: {}, Disk: {}, Inner: {}", block, block_offset, disk_index, inner);
        }

        // let mut i = 0;
        // while {
            
        //     for block in 0..self.block_size {
        //         let block_index = i + block;
        //         println!("Block Index: {}", block);

        //         // let block_index = (self.block_size * (i + current_offset)) + block;

        //         // let disk = block_index % self.array.len();
        //         // let disk_level_offset = block_index / self.array.len();
        //         // println!("Block index: {block_index}, Disk: {disk}, Offset: {disk_level_offset}");
        //     }


            
        //     // self.array[disk].write(RawStoragePtr::byte_ptr((i + current_offset) / self.array.len()), &[data[i]]).get();
        //     i += 1
        // }
        self.offset.fetch_add(data.len(), Ordering::SeqCst);
        ptr
    }
    /// Reads from the RAID4 array. Only supports byte level reads.
    pub fn read(&self, ptr: RawStoragePtr, length: usize) -> Vec<u8> {
        let mut buffer = vec![];


        for i in ptr.byte_offset..ptr.byte_offset + length {
            let block = (i + ptr.byte_offset) / self.block_size;
            let disk_index = block % self.array.len();
            let inner = block + ((i - (block * self.block_size)) % self.block_size);
            
            buffer.push(self.array[disk_index].read(RawStoragePtr::byte_ptr(inner), length).get()[0]);
        }

        // for i in (ptr.byte_offset)..(ptr.byte_offset + length) {
        //     buffer.push(self.array[i % self.array.len()].read(RawStoragePtr::byte_ptr(i / self.array.len()), 1).get()[0]);
        // }
        buffer
    }
    // /// Checks the integrity of the RAID4 array.
    // pub fn check_array_integrity(&self) -> bool {
    //     let current_offset = self.offset.load(Ordering::SeqCst);
    //     for byte in 0..current_offset {
    //         let parity = self.parity.read_bit(RawStoragePtr::bit_ptr(byte)).get();
    //         let current: BitVec = self.read(RawStoragePtr::byte_ptr(byte), 1)[0].into();
      
    //         if parity != current.parity() {
    //             return false;
    //         }
    //     }
    //     true
    // }
}

#[cfg(test)]
mod tests {
    use crate::disks::{hard_drive::{DiskAlgorithm, MagneticDisk}, raid::{raid4::Raid4Builder, raid5::Raid5Builder}, AbstractStorageDevice, RawStoragePtr};



    #[test]
    pub fn test_raid5_array() {
        let raid = Raid5Builder::default()
            .with_disk(MagneticDisk::new(256, DiskAlgorithm::FCFS))
            .with_disk(MagneticDisk::new(256, DiskAlgorithm::FCFS))
            // .with_disk(MagneticDisk::new(256, DiskAlgorithm::FCFS))
            .with_block_size(2)
            .build();

        
        // Do a write and read it
        let ptr = raid.write(&[2, 4, 5, 6, 7, 8, 9, 10, 11]);
        assert_eq!(raid.read(ptr, 3), [2, 4, 5, 6, 7, 8, 9, 10, 11]);

     

        // assert!(raid.check_array_integrity());

        // // println!("\n\nDONE\n\n");


        // // read another write.
        // let ptr2 = raid.write(&[6,7,8,9, 10, 11, 12, 13]);
        // assert_eq!(raid.read(ptr2, 4), [6,7,8,9]);

        panic!("wow");
        // // check integiry
        // assert!(raid.check_array_integrity());
  
    }

    // #[test]
    // pub fn test_raid4_array_corruption() {
    //     let raid = Raid4Builder::default()
    //         .with_disk(MagneticDisk::new(256, DiskAlgorithm::FCFS))
    //         .with_disk(MagneticDisk::new(256, DiskAlgorithm::FCFS))
    //         .with_disk(MagneticDisk::new(256, DiskAlgorithm::FCFS))
    //         .with_parity_disk(MagneticDisk::new(256, DiskAlgorithm::FCFS))
    //         .build();

        
        
    //     // Do a write and read it
    //     let ptr = raid.write(&[2, 4, 5]);
    //     assert_eq!(raid.read(ptr, 3), [2, 4, 5]);


    //     // Corrupt the array
    //     raid.array[0].write_bit(RawStoragePtr::bit_ptr(0), true);

    

    //     // Check integrity, this should detect
    //     // that it is corrupted.
    //     assert!(!raid.check_array_integrity());
  
    // }
}