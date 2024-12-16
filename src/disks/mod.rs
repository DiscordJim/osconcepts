use crate::memory::ipc::Yield;


pub mod hard_drive;
pub mod raid;
pub mod bits;

pub type Bit = bool;

#[derive(Clone, Debug, Copy)]
pub struct RawStoragePtr {
    byte_offset: usize,
    bit_offset: u8
}

impl RawStoragePtr {
    pub fn bit_ptr(bit_pos: usize) -> Self {
        Self {
            byte_offset: bit_pos / 8,
            bit_offset: (bit_pos % 8).try_into().unwrap()
        }
    }
    pub fn byte_ptr(byte_offset: usize) -> Self {
        Self {
            byte_offset,
            bit_offset: 0
        }
    }
}

pub trait StorageDevice {
    fn write_bit(&mut self, addr: RawStoragePtr, bit: Bit);
    fn read_bit(&mut self, addr: RawStoragePtr) -> Bit;
    fn store(&mut self, data: &[u8]) -> RawStoragePtr;
    fn write(&mut self, addr: RawStoragePtr, data: &[u8]);
    fn read(&self, addr: RawStoragePtr, length: usize) -> Vec<u8>;
}

pub trait AbstractStorageDevice {
    fn write_bit(&self, addr: RawStoragePtr, bit: Bit) -> Yield<()>;
    fn read_bit(&self, addr: RawStoragePtr) -> Yield<Bit>;
    fn store(&self, data: &[u8]) -> Yield<RawStoragePtr>;
    fn write(&self, addr: RawStoragePtr, data: &[u8]) -> Yield<()>;
    fn read(&self, addr: RawStoragePtr, length: usize) -> Yield<Vec<u8>>;
}

#[derive(Default)]
pub struct SecondaryStorage {
    buffer: Vec<u8>,
    offset: usize,
}

impl SecondaryStorage {
    pub fn new(size: usize) -> Self {
        Self {
            buffer: vec![0u8; size],
            offset: 0
        }
    }
    pub fn get_offset(&self) -> usize {
        self.offset
    }
    
}

impl StorageDevice for SecondaryStorage {
    fn read_bit(&mut self, addr: RawStoragePtr) -> Bit {
        ((self.buffer[addr.byte_offset] >> (7 - addr.bit_offset)) & 1) != 0
    }
    fn write_bit(&mut self, addr: RawStoragePtr, bit: Bit) {
        if bit {
            self.buffer[addr.byte_offset] |= 1 << (7 - addr.bit_offset);
        } else {
            self.buffer[addr.byte_offset] &= !(1 << (7 - addr.bit_offset));
        }


        // let related_bytes = self.buffer[addr.byte_offset];
        

        // let mut cand = 2;

        // let pos = 1;
        // cand = cand | (1 << (7 - pos));

        // println!("related bytes {:#010b}", self.buffer[addr.byte_offset]);

        // for i in 8..0 {
        //     let t = cand >> i & 1;
        //     print!("{t}");
        //     if i > 0 && (i + 1) % 4 == 0 {
        //         print!(" ");
        //     }
        // }
        // println!();
        
    }
    fn store(&mut self, data: &[u8]) -> RawStoragePtr {
        let addr = self.offset;

        self.buffer[addr..addr + data.len()].copy_from_slice(data);

        // for i in 0..data.len() {
        //     self.buffer[addr + i] = data[i];
        // }

        self.offset += data.len();
        RawStoragePtr {
            byte_offset: addr,
            bit_offset: 0
        }
    }
    fn write(&mut self, addr: RawStoragePtr, data: &[u8]) {
        self.buffer[addr.byte_offset..addr.byte_offset + data.len()].copy_from_slice(data);
     
    }
    fn read(&self, addr: RawStoragePtr, length: usize) -> Vec<u8> {
        
        self.buffer[addr.byte_offset..addr.byte_offset + length].to_vec()
    }
}

#[cfg(test)]
mod tests {
    use crate::disks::StorageDevice;

    use super::{RawStoragePtr, SecondaryStorage};


    #[test]
    pub fn test_storage() {
        let mut disk = SecondaryStorage::new(8096);

        let addr = disk.store(&[1,2,3]);
        assert_eq!(disk.read(addr, 3), [1,2,3]);
        disk.write(addr, &[3,4]);
        assert_eq!(disk.read(addr, 3), [3,4,3]);
    }

    #[test]
    pub fn test_storage_bit() {
        let mut disk = SecondaryStorage::new(8096);
        disk.write_bit(RawStoragePtr::bit_ptr(3), true);
        assert!(disk.read_bit(RawStoragePtr::bit_ptr(3)));
        disk.write_bit(RawStoragePtr::bit_ptr(2), true);
        assert!(disk.read_bit(RawStoragePtr::bit_ptr(3)));
        disk.write_bit(RawStoragePtr::bit_ptr(3), false);
        assert!(!disk.read_bit(RawStoragePtr::bit_ptr(3)));
        assert!(disk.read_bit(RawStoragePtr::bit_ptr(2)));
        // assert!(disk.read_bit(RawStoragePtr::bit_ptr(0)));
        
    }
}
