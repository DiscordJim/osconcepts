use std::sync::atomic::{AtomicUsize, Ordering};

use crate::disks::{bits::BitVec, hard_drive::MagneticDisk, AbstractStorageDevice, RawStoragePtr};


#[derive(Default)]
pub struct Raid3Builder {
    array: Vec<MagneticDisk>,
    parity: Option<MagneticDisk>
}

impl Raid3Builder {
    pub fn with_disk(mut self, disk: MagneticDisk) -> Self {
        self.array.push(disk);
        self
    }
    pub fn with_parity_disk(mut self, disk: MagneticDisk) -> Self {
        self.parity = Some(disk);
        self
    }
    pub fn build(self) -> Raid3 {
        Raid3 {
            array: self.array.into_boxed_slice(),
            parity: self.parity.unwrap(),
            offset: AtomicUsize::new(0)
        }
    }
}

/// A RAID3 array. Mirroring.
pub struct Raid3 {
    array: Box<[MagneticDisk]>,
    parity: MagneticDisk,
    offset: AtomicUsize
}

impl Raid3 {
    /// Writes to the RAID3 array, performing striping
    /// at the byte level.
    /// 
    /// This will only perform byte level writes.
    pub fn write(&self, data: &[u8]) -> RawStoragePtr {
        let current_offset = self.offset.load(Ordering::SeqCst);
        let ptr = RawStoragePtr::byte_ptr(current_offset);

        for i in 0..data.len() {
            let byte = BitVec::from(data[i]);
            for j in 0..8 {
                let bit_index = (8 * (i + current_offset)) + j;

                let disk = bit_index % self.array.len();
                let bit_height = bit_index / self.array.len();

                self.array[disk].write_bit(RawStoragePtr::bit_ptr(bit_height), byte[j]);
            }

            // Write the parity byte to the disk.
            self.parity.write_bit(RawStoragePtr::bit_ptr(i + current_offset), byte.parity());
        }
        self.offset.fetch_add(data.len(), Ordering::SeqCst);
        ptr
    }
    /// Reads from the RAID3 array. Only supports byte level reads.
    pub fn read(&self, ptr: RawStoragePtr, length: usize) -> Vec<u8> {
        let mut buffer = vec![0u8; length];
        for bit_index in ptr.byte_offset * 8..((ptr.byte_offset + length) * 8) {
         

            // Get the index into the buffer.
            let buffer_index = (bit_index - (8 * ptr.byte_offset)) / 8;
            let buffer_stride = (bit_index - (8 * ptr.byte_offset)) % 8;
            
            // This calcuates what disk we have to look up from and the position of the bi.
            let disk = bit_index % self.array.len();
            let bit_height = bit_index / self.array.len();

            // Updates the value in the buffer.
            if self.array[disk].read_bit(RawStoragePtr::bit_ptr(bit_height)).get() {
                buffer[buffer_index] |= 1 << (7 - buffer_stride);
            }
        }
        buffer
    }
    /// Checks the integrity of the RAID3 array.
    pub fn check_array_integrity(&self) -> bool {
        let current_offset = self.offset.load(Ordering::SeqCst);
        for byte in 0..current_offset {
            let parity = self.parity.read_bit(RawStoragePtr::bit_ptr(byte)).get();
            let current: BitVec = self.read(RawStoragePtr::byte_ptr(byte), 1)[0].into();
      
            if parity != current.parity() {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::disks::{hard_drive::{DiskAlgorithm, MagneticDisk}, raid::raid3::Raid3Builder, AbstractStorageDevice, RawStoragePtr};



    #[test]
    pub fn test_raid3_array() {
        let raid = Raid3Builder::default()
            .with_disk(MagneticDisk::new(256, DiskAlgorithm::FCFS))
            .with_disk(MagneticDisk::new(256, DiskAlgorithm::FCFS))
            .with_disk(MagneticDisk::new(256, DiskAlgorithm::FCFS))
            .with_parity_disk(MagneticDisk::new(256, DiskAlgorithm::FCFS))
            .build();

        
        // Do a write and read it
        let ptr = raid.write(&[2, 4, 5]);
        assert_eq!(raid.read(ptr, 3), [2, 4, 5]);

        assert!(raid.check_array_integrity());

        // println!("\n\nDONE\n\n");


        // read another write.
        let ptr2 = raid.write(&[6,7,8,9]);
        assert_eq!(raid.read(ptr2, 4), [6,7,8,9]);

        // check integiry
        assert!(raid.check_array_integrity());
  
    }

    #[test]
    pub fn test_raid3_array_corruption() {
        let raid = Raid3Builder::default()
            .with_disk(MagneticDisk::new(256, DiskAlgorithm::FCFS))
            .with_disk(MagneticDisk::new(256, DiskAlgorithm::FCFS))
            .with_disk(MagneticDisk::new(256, DiskAlgorithm::FCFS))
            .with_parity_disk(MagneticDisk::new(256, DiskAlgorithm::FCFS))
            .build();

        
        
        // Do a write and read it
        let ptr = raid.write(&[2, 4, 5]);
        assert_eq!(raid.read(ptr, 3), [2, 4, 5]);


        // Corrupt the array
        raid.array[0].write_bit(RawStoragePtr::bit_ptr(0), true);

    

        // Check integrity, this should detect
        // that it is corrupted.
        assert!(!raid.check_array_integrity());
  
    }
}