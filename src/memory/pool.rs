use std::{collections::HashMap, fmt::Debug, marker::PhantomData, mem, ops::{Deref, DerefMut}, sync::{Arc, Weak}};

use parking_lot::{ArcMutexGuard, Mutex, RawMutex};

use super::SharedMemory;


type RamMap = SharedMemory<HashMap<u32, Box<[u8]>>>;

pub struct MemoryRaw;
pub struct MemoryMutex;

pub enum MemoryProtection {
    /// No memory protections
    Raw,
    /// Mutex
    Mutex
}

/// the most unsafe thing ever implemented.
pub struct RandomAccessMemory<P> {
    lookup: Arc<RamMap>,
    guard: Arc<Mutex<()>>,
    _mode: PhantomData<P>
}

impl<P> RandomAccessMemory<P> {
    pub fn new() -> Self {
        Self {
            lookup: Arc::new(SharedMemory::new(HashMap::new())),
            guard: Arc::new(Mutex::new(())),
            _mode: PhantomData
        }
    }
    fn store_inner<T>(&self, obj: T) -> MemoryPtr<T> {
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
            shr_guard_mutex: Arc::downgrade(&self.guard),
            _type: PhantomData
        }
    }
}

impl RandomAccessMemory<MemoryMutex> {
    pub fn store<T>(&self, object: T) -> SyncMemoryPtr<T> {
        SyncMemoryPtr(self.store_inner(object))
    }
}

impl RandomAccessMemory<MemoryRaw> {
    pub fn store<T>(&self, object: T) -> MemoryPtr<T> {
        self.store_inner(object)
    }
}

pub struct MemoryPtr<T> {
    address: u32,
    ram: Weak<RamMap>,
    shr_guard_mutex: Weak<Mutex<()>>,
    _type: PhantomData<T>
}


pub struct SyncMemoryPtr<T>(MemoryPtr<T>);

impl<T> Clone for SyncMemoryPtr<T> {
    fn clone(&self) -> Self {
        Self(MemoryPtr::clone(&self.0))
    }
}

impl<T> SyncMemoryPtr<T> {
    pub fn lock<'a>(&self) -> MemoryPtrGuard<T> {
        let mutex_arc = self.0.shr_guard_mutex.upgrade().unwrap().clone();
        let guard= mutex_arc.lock_arc();

        MemoryPtrGuard {
            ptr: self.0.clone(),
            _guard: guard
        }
    } 
}

impl<T: Debug>  Debug for SyncMemoryPtr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub struct MemoryPtrGuard<T> {
   ptr: MemoryPtr<T>,
   _guard: ArcMutexGuard<RawMutex, ()> 
}  



impl<T> MemoryPtrGuard<T> {
    pub fn get(&self) -> &T {
        self.ptr.get()
    }
    pub fn get_mut(&self) -> &mut T {
        self.ptr.get_mut()
    }
}


impl<T: Debug> Debug for MemoryPtr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.get().fmt(f)
    }
}

unsafe impl<T> Send for MemoryPtr<T> {}
unsafe impl<T> Sync for MemoryPtr<T> {}

impl<T> Clone for MemoryPtr<T> {
    fn clone(&self) -> Self {
        Self {
            address: self.address,
            ram: Weak::clone(&self.ram),
            shr_guard_mutex: Weak::clone(&self.shr_guard_mutex),
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
    use crate::memory::pool::MemoryRaw;

    use super::RandomAccessMemory;

    #[derive(Debug)]
    pub struct TestStub {
        a: u16
    }

    #[test]
    pub fn test_random_anonymous_map() {
        let ram = RandomAccessMemory::<MemoryRaw>::new();

        let data = TestStub {
            a: 3
        };

 
        let ptr = ram.store(data);


        assert_eq!(ptr.a, 3);

        ptr.get_mut().a = 24;
        
       
        assert_eq!(ptr.a, 24);

        
    }
}