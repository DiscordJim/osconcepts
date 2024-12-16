use std::{marker::PhantomData, sync::Arc};

use parking_lot::Mutex;

use crate::{memory::ipc::IpcChannel, Delay};

use super::{RawStoragePtr, SecondaryStorage, StorageDevice};

enum ServiceRequest {
    Read {
        addr: RawStoragePtr,
        outbound: Arc<IpcChannel<Vec<u8>>>,
    },
    Write {
        bytes: Vec<u8>,
        inbound: Arc<IpcChannel<RawStoragePtr>>,
    },
    Edit {
        pointer: RawStoragePtr,
        data: Vec<u8>,
        confirm: Arc<IpcChannel<()>>
    }
}


pub struct MagneticDisk {
   
    requests: Arc<IpcChannel<ServiceRequest>>,
}



impl MagneticDisk {
    pub fn new() -> Self {
        let object = Self {
            requests: Arc::new(IpcChannel::new()),
        };
        std::thread::spawn({
            let requests = Arc::clone(&object.requests);
            move || {
                run_disk(requests, SecondaryStorage::new());
            }
        });
        object
    }
    
}


fn run_disk(request_queue: Arc<IpcChannel<ServiceRequest>>, mut storage: SecondaryStorage) {
    loop {
        let item = request_queue.recv();
        match item {
            ServiceRequest::Read { addr, outbound } => {
                outbound.send(storage.read(addr).to_vec());
            },
            ServiceRequest::Edit { pointer, data, confirm } => {
                storage.write(pointer, &data);
                confirm.send(());
            },
            ServiceRequest::Write { bytes, inbound } => {
                inbound.send(storage.store(&bytes));
            }
        }
    }
}

impl StorageDevice for MagneticDisk {
    fn write(&mut self, addr: super::RawStoragePtr, data: &[u8]) {
        let chan = Arc::new(IpcChannel::new());
        let request = ServiceRequest::Edit {
            pointer: addr,
            data: data.to_vec(),
            confirm: chan.clone(),
        };
        self.requests.send(request);
        chan.recv()
    }
    fn read(&self, addr: super::RawStoragePtr) -> Vec<u8> {
        let chan = Arc::new(IpcChannel::new());
        let request = ServiceRequest::Read {
            addr,
            outbound: chan.clone(),
        };
        self.requests.send(request);
        chan.recv()
    }
    fn store(&mut self, data: &[u8]) -> super::RawStoragePtr {
        let chan = Arc::new(IpcChannel::new());
        let request = ServiceRequest::Write {
            bytes: data.to_vec(),
            inbound: chan.clone(),
        };
        self.requests.send(request);
        chan.recv()
    }
}


#[cfg(test)]
mod tests {
    use crate::disks::{RawStoragePtr, StorageDevice};

    use super::MagneticDisk;


    #[test]
    pub fn test_magnetic_disk() {

        let mut magn = MagneticDisk::new();
        let wow = magn.store(&[1,2,3]);
        assert_eq!(magn.read(wow), [1,2,3]);
        
    }
}