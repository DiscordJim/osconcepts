use std::collections::VecDeque;

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