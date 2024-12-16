use std::sync::atomic::{AtomicUsize, Ordering};

use crate::disks::{hard_drive::MagneticDisk, AbstractStorageDevice, RawStoragePtr};

/// A RAID0 array. Stripping is done at the byte level for simplicity.
pub struct Raid0 {
    array: Vec<MagneticDisk>,
    offset: AtomicUsize
}

impl Raid0 {
    pub fn new() -> Self {
        Self {
            array: vec![],
            offset: AtomicUsize::new(0)
        }
    }
    pub fn with_disk(mut self, disk: MagneticDisk) -> Self {
        self.array.push(disk);
        self
    }
    /// Writes to the RAID0 array, performing striping
    /// at the byte level.
    pub fn write(&self, data: &[u8]) -> RawStoragePtr {
        let ptr = RawStoragePtr::byte_ptr(self.offset.load(Ordering::SeqCst));
        let current_offset = self.offset.load(Ordering::SeqCst);
        for i in 0..data.len() {
            let disk = (i + current_offset) % self.array.len();
            self.array[disk].write(RawStoragePtr::byte_ptr((i + current_offset) / self.array.len()), &[data[i]]).get();
        }
        self.offset.fetch_add(data.len(), Ordering::SeqCst);
        ptr
    }
    /// Reads from the RAID0 array.
    pub fn read(&self, ptr: RawStoragePtr, length: usize) -> Vec<u8> {
        let mut buffer = vec![];
        for i in (ptr.byte_offset)..(ptr.byte_offset + length) {
            buffer.push(self.array[i % self.array.len()].read(RawStoragePtr::byte_ptr(i / self.array.len()), 1).get()[0]);
        }
        buffer
    }
}

#[cfg(test)]
mod tests {
    use crate::disks::hard_drive::{DiskAlgorithm, MagneticDisk};

    use super::Raid0;

    #[test]
    pub fn test_raid0_array() {
        let raid = Raid0::new()
            .with_disk(MagneticDisk::new(256, DiskAlgorithm::FCFS))
            .with_disk(MagneticDisk::new(256, DiskAlgorithm::FCFS))
            .with_disk(MagneticDisk::new(256, DiskAlgorithm::FCFS));

        
        // Do a write and read it
        let ptr = raid.write(&[1,2,3,4,5]);
        assert_eq!(raid.read(ptr, 5), [1,2,3,4,5]);


        // read another write.
        let ptr2 = raid.write(&[6,7,8,9]);
        assert_eq!(raid.read(ptr2, 4), [6,7,8,9]);
  
    }
}