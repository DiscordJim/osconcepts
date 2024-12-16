use std::sync::atomic::{AtomicUsize, Ordering};

use crate::disks::{bits::BitVec, hard_drive::MagneticDisk, AbstractStorageDevice, RawStoragePtr};

/// A RAID3 array. Mirroring.
pub struct Raid3 {
    array: Vec<MagneticDisk>,
    offset: AtomicUsize
}

impl Raid3 {
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
        let current_offset = self.offset.load(Ordering::SeqCst);
        let ptr = RawStoragePtr::byte_ptr(current_offset / 8);

        for i in 0..data.len() {
            let byte = BitVec::from(data[i]);
            for j in 0..8 {
                let bit_index = (8 * (i + current_offset)) + j;

                let disk = bit_index % self.array.len();
                let bit_height = bit_index / self.array.len();

                self.array[disk].write_bit(RawStoragePtr::bit_ptr(bit_height), byte[j]);
                



                // println!("{} ({}) ({disk}) ", byte[j], bit_index);
            }
            println!();
            
        }

        // for disk in &self.array {
        //     disk.write(ptr, data);
        // }
        // self.offset.fetch_add(data.len(), Ordering::SeqCst);
        ptr
    }
    /// Reads from the RAID3 array. Only supports byte level reads.
    pub fn read(&self, ptr: RawStoragePtr, length: usize) -> Vec<u8> {
        let mut buffer = vec![0u8; length];
        for i in ptr.byte_offset * 8..((ptr.byte_offset + length) * 8) {
            // Get the bit index
            let bit_index = i + (8 * ptr.byte_offset);

            // Get the index into the buffer.
            let buffer_index = (i - (8 * ptr.byte_offset)) / 8;
            let buffer_stride = (i - (8 * ptr.byte_offset)) % 8;
            
            // This calcuates what disk we have to look up from and the position of the bi.
            let disk = bit_index % self.array.len();
            let bit_height = bit_index / self.array.len();

            if self.array[disk].read_bit(RawStoragePtr::bit_ptr(bit_height)).get() {
                buffer[buffer_index] |= 1 << (7 - buffer_stride);
            }
            // println!("Bit: {bit}");



        }
        buffer
        // self.array[0].read(ptr, length).get()
    }
}

#[cfg(test)]
mod tests {
    use crate::disks::hard_drive::{DiskAlgorithm, MagneticDisk};

    use super::Raid3;

    #[test]
    pub fn test_raid1_array() {
        let raid = Raid3::new()
            .with_disk(MagneticDisk::new(256, DiskAlgorithm::FCFS))
            .with_disk(MagneticDisk::new(256, DiskAlgorithm::FCFS))
            .with_disk(MagneticDisk::new(256, DiskAlgorithm::FCFS));

        
        // Do a write and read it
        let ptr = raid.write(&[2, 4, 5]);
        assert_eq!(raid.read(ptr, 3), [2, 4, 5]);


        // read another write.
        let ptr2 = raid.write(&[6,7,8,9]);
        assert_eq!(raid.read(ptr2, 4), [6,7,8,9]);
  
    }
}