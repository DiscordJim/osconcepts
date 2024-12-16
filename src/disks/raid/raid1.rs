use std::sync::atomic::{AtomicUsize, Ordering};

use crate::disks::{hard_drive::MagneticDisk, AbstractStorageDevice, RawStoragePtr};

/// A RAID1 array. Mirroring.
pub struct Raid1 {
    array: Vec<MagneticDisk>,
    offset: AtomicUsize
}

impl Raid1 {
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
        for disk in &self.array {
            disk.write(ptr, data);
        }
        self.offset.fetch_add(data.len(), Ordering::SeqCst);
        ptr
    }
    /// Reads from the RAID0 array.
    pub fn read(&self, ptr: RawStoragePtr, length: usize) -> Vec<u8> {
        self.array[0].read(ptr, length).get()
    }
}

#[cfg(test)]
mod tests {
    use crate::disks::hard_drive::{DiskAlgorithm, MagneticDisk};

    use super::Raid1;

    #[test]
    pub fn test_raid1_array() {
        let raid = Raid1::new()
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