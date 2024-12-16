use std::collections::HashMap;

use rand::random;

pub mod hard_drive;

#[derive(Clone, Debug, Copy)]
pub struct RawStoragePtr(u64);

pub trait StorageDevice {
    fn store(&mut self, data: &[u8]) -> RawStoragePtr;
    fn write(&mut self, addr: RawStoragePtr, data: &[u8]);
    fn read(&self, addr: RawStoragePtr) -> Vec<u8>;
}


#[derive(Default)]
pub struct SecondaryStorage {
    map: HashMap<u64, Box<[u8]>>
}

impl SecondaryStorage {
    pub fn new() -> Self {
        Self {
            map: HashMap::default()
        }
    }
    
}

impl StorageDevice for SecondaryStorage {
    fn store(&mut self, data: &[u8]) -> RawStoragePtr {
        let addr = random();
        self.map.insert(addr, data.to_vec().into_boxed_slice());
        RawStoragePtr(addr)
    }
    fn write(&mut self, addr: RawStoragePtr, data: &[u8]) {
        if !self.map.contains_key(&addr.0) {
            panic!("Could not find address.");
        }
        self.map.insert(addr.0, data.to_vec().into_boxed_slice());
    }
    fn read(&self, addr: RawStoragePtr) -> Vec<u8> {
        self.map.get(&addr.0).unwrap().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use crate::disks::StorageDevice;

    use super::SecondaryStorage;


    #[test]
    pub fn test_storage() {
        let mut disk = SecondaryStorage::new();

        let addr = disk.store(&[1,2,3]);
        assert_eq!(disk.read(addr), [1,2,3]);
        disk.write(addr, &[1,2]);
        assert_eq!(disk.read(addr), [1,2]);
    }
}
