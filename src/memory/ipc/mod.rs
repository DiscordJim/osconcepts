use std::{collections::VecDeque, sync::Arc};

use parking_lot::{Condvar, Mutex};



pub struct IpcChannel<T> {
    signal: Condvar,
    queue: Mutex<VecDeque<T>>
}

impl<T> IpcChannel<T> {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            signal: Condvar::new(),
        }
    }
    /// Sends a value into the IPC channel.
    pub fn send(&self, data: T) {
        let mut queue = self.queue.lock();
        queue.push_back(data);
        self.signal.notify_one();
    }
    /// Tries to receive a value.
    pub fn try_recv(&self) -> Option<T> {
        let mut queue = self.queue.lock();
        if !queue.is_empty() {
            queue.pop_front()
        } else {
            None
        }
    }
    /// Receives unconditionally.
    pub fn recv(&self) -> T {
        let mut queue = self.queue.lock();
        if !queue.is_empty() {
            queue.pop_front().unwrap()
        } else {
            self.signal.wait(&mut queue);
            queue.pop_front().unwrap()
        }
    }
}


/// A synchronous future.
pub struct Yield<T>(Arc<IpcChannel<T>>);

impl<T> Yield<T> {
    pub fn new(channel: Arc<IpcChannel<T>>) -> Self {
        Self(channel)
    }
    pub fn get(self) -> T {
        self.0.recv()
    }
    pub fn join_get(mut yields: Vec<Yield<T>>) {
        for _ in 0..yields.len() {
            yields.pop().unwrap().get();
        }
    }
}