use std::{collections::HashMap, marker::PhantomData, mem, ops::{Deref, DerefMut}, sync::{Arc, Weak}};

use super::SharedMemory;


type RamMap = SharedMemory<HashMap<u32, Box<[u8]>>>;

/// the most unsafe thing ever implemented.
pub struct RandomAccessMemory {
    lookup: Arc<RamMap>
}

impl RandomAccessMemory {
    pub fn new() -> Self {
        Self {
            lookup: Arc::new(SharedMemory::new(HashMap::new()))
        }
    }
    pub fn store<T: std::fmt::Debug>(&self, obj: T) -> MemoryPtr<T> {
        let rand = rand::random::<u32>();

        // Store this object as raw bytes on the heap.
        let structdat = unsafe {core::slice::from_raw_parts(
            (&obj as *const T) as *const u8,
            ::core::mem::size_of::<T>(),
        ) }.to_vec().into_boxed_slice();

        // don't let the destructor run
        mem::forget(obj);


        // insert this in the map.
        self.lookup.get_mut().insert(rand, structdat);

   
        MemoryPtr {
            address: rand,
            ram: Arc::downgrade(&self.lookup),
            _type: PhantomData
        }
    }
}


pub struct MemoryPtr<T> {
    address: u32,
    ram: Weak<RamMap>,
    _type: PhantomData<T>
}

impl<T> Clone for MemoryPtr<T> {
    fn clone(&self) -> Self {
        Self {
            address: self.address,
            ram: Weak::clone(&self.ram),
            _type: PhantomData
        }
    }
}





impl<T> MemoryPtr<T> {
    fn get_raw(&self) -> *const [u8] {
        if let Some(map) = self.ram.upgrade() {
            let actual_ptr = map.get().get(&self.address).unwrap().as_ref() as *const [u8];
            actual_ptr
        } else {
            panic!("Deref to null ram!");
        }
    }
    pub fn get(&self) -> &T {
        let ptr = unsafe { &*self.get_raw() };
        let (_, body, _) = unsafe { ptr.align_to::<T>() };
        &body[0]
    }
    pub fn get_mut(&self) -> &mut T {
        let ptr = unsafe { &mut *(self.get_raw() as *mut [u8]) };
        let (_, body, _) = unsafe { ptr.align_to_mut::<T>() };
        &mut body[0]
    }
    // pub fn get_mut(&self) -> &mut T {
    //     let bytes = unsafe { &mut *(self.get_raw() as *mut [u8]) };
    //     let (_, body, _) = unsafe { bytes.align_to_mut::<T>() };
    //     &mut body[0]
    // }
}

impl<T> Deref for MemoryPtr<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T> DerefMut for MemoryPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}


#[cfg(test)]
mod tests {
    use super::RandomAccessMemory;

    #[derive(Debug)]
    pub struct TestStub {
        a: u16
    }

    #[test]
    pub fn test_random_anonymous_map() {
        let ram = RandomAccessMemory::new();

        let data = TestStub {
            a: 3
        };

 
        let ptr = ram.store(data);


        assert_eq!(ptr.a, 3);

        ptr.get_mut().a = 24;
        
       
        assert_eq!(ptr.a, 24);

        
    }
}