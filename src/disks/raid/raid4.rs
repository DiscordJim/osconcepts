use std::sync::atomic::{AtomicUsize, Ordering};

use crate::disks::{bits::BitVec, hard_drive::MagneticDisk, AbstractStorageDevice, RawStoragePtr};


#[derive(Default)]
pub struct Raid4Builder {
    array: Vec<MagneticDisk>,
    parity: Option<MagneticDisk>
}

impl Raid4Builder {
    pub fn with_disk(mut self, disk: MagneticDisk) -> Self {
        self.array.push(disk);
        self
    }
    pub fn with_parity_disk(mut self, disk: MagneticDisk) -> Self {
        self.parity = Some(disk);
        self
    }
    pub fn build(self) -> Raid4 {
        Raid4 {
            array: self.array.into_boxed_slice(),
            parity: self.parity.unwrap(),
            offset: AtomicUsize::new(0)
        }
    }
}

/// A RAID4 array. 
/// 
/// Byte-level (it should be block but this is a huge pain so we are doing it at
/// the byte level) striping w/ a parity disk.
pub struct Raid4 {
    array: Box<[MagneticDisk]>,
    parity: MagneticDisk,
    offset: AtomicUsize
}

impl Raid4 {
    /// Writes to the RAID4 array, performing striping at the
    /// byte level.
    /// 
    /// This will only perform byte level writes.
    pub fn write(&self, data: &[u8]) -> RawStoragePtr {
        let ptr = RawStoragePtr::byte_ptr(self.offset.load(Ordering::SeqCst));
        let current_offset = self.offset.load(Ordering::SeqCst);
        for i in 0..data.len() {
            let disk = (i + current_offset) % self.array.len();
            self.array[disk].write(RawStoragePtr::byte_ptr((i + current_offset) / self.array.len()), &[data[i]]).get();
            self.parity.write_bit(RawStoragePtr::bit_ptr(i + current_offset), BitVec::from(data[i]).parity()).get();
        }
        self.offset.fetch_add(data.len(), Ordering::SeqCst);
        ptr
    }
    /// Reads from the RAID4 array. Only supports byte level reads.
    pub fn read(&self, ptr: RawStoragePtr, length: usize) -> Vec<u8> {
        let mut buffer = vec![];
        for i in (ptr.byte_offset)..(ptr.byte_offset + length) {
            buffer.push(self.array[i % self.array.len()].read(RawStoragePtr::byte_ptr(i / self.array.len()), 1).get()[0]);
        }
        buffer
    }
    /// Checks the integrity of the RAID4 array.
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
    use crate::disks::{hard_drive::{DiskAlgorithm, MagneticDisk}, raid::raid4::Raid4Builder, AbstractStorageDevice, RawStoragePtr};



    #[test]
    pub fn test_raid4_array() {
        let raid = Raid4Builder::default()
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
    pub fn test_raid4_array_corruption() {
        let raid = Raid4Builder::default()
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