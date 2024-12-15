use std::{cell::UnsafeCell, sync::Arc};

pub mod pool;

/// For unsafe unguarded memory sharing between threads.
pub struct SharedMemory<T>(Arc<UnsafeCell<T>>);

impl<T> SharedMemory<T> {
    pub fn new(obj: T) -> Self {
        Self(Arc::new(UnsafeCell::new(obj)))
    }
}

impl<T> Clone for SharedMemory<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T> SharedMemory<T> {
    pub fn get(&self) -> &T {
        unsafe { &*self.0.get() } 
    }
    pub fn get_mut(&self) -> &mut T {
        unsafe { &mut *self.0.get() }
    }
}

// impl<T> Deref for SharedMemory<T> {
//     type Target = T;
//     fn deref(&self) -> &Self::Target {
//         unsafe { &*self.0.get() }
//     }
// }

// impl<T> DerefMut for SharedMemory<T> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         unsafe { &mut *self.0.get() }
//     }
// }