use std::{
    sync::{
        atomic::{AtomicU8, AtomicUsize, Ordering},
        Arc,
    },
    thread::yield_now,
};

use parking_lot::Mutex;

use crate::memory::ipc::{IpcChannel, Yield};

use super::{AbstractStorageDevice, Bit, RawStoragePtr, SecondaryStorage, StorageDevice};

#[derive(PartialEq)]
pub enum DiskAlgorithm {
    /// Serves requests in a first come first served
    /// manner.
    FCFS,
    /// Serves requests as the shortest seek time next.
    SSTF,
    /// Scans across the disk servicing requests as we go,
    /// once we reach the end we go back and service request.
    SCAN,
    /// Scans across the disk but unlike SCAN, when we reach
    /// the end we just jump back to the beginning.
    CSCAN,
    /// CSCAN but jumps to the beginning.
    CLOOK
}

pub enum ServiceRequest {
    Read {
        addr: RawStoragePtr,
        outbound: Arc<IpcChannel<Vec<u8>>>,
        length: usize,
    },
    Write {
        bytes: Vec<u8>,
        inbound: Arc<IpcChannel<RawStoragePtr>>,
    },
    ReadBit {
        addr: RawStoragePtr,
        outbound: Arc<IpcChannel<bool>>,
    },
    WriteBit {
        addr: RawStoragePtr,
        value: Bit,
        confirm: Arc<IpcChannel<()>>,
    },
    Edit {
        addr: RawStoragePtr,
        data: Vec<u8>,
        confirm: Arc<IpcChannel<()>>,
    }
}

pub struct MagneticDisk {
    /// All the scheduled service rquests.
    requests: Arc<IpcChannel<ServiceRequest>>,

    /// Keeps track of all the requests serviced, mostly
    /// used for testing.
    service_record: Arc<Mutex<Vec<usize>>>,


    offset: Arc<AtomicUsize>,

    /// The states are as follows,
    /// 0 = Paused
    /// 1 = Running
    /// 2 = Shutdown
    state: Arc<AtomicU8>,
}

impl MagneticDisk {
    pub fn new(size: usize, algorithm: DiskAlgorithm) -> Self {
        let object = Self {
            requests: Arc::new(IpcChannel::new()),
            state: Arc::new(AtomicU8::new(1)),
            offset: Arc::new(AtomicUsize::new(0)),
            service_record: Arc::default(),
        };
        std::thread::spawn({
            let requests = Arc::clone(&object.requests);
            let state = Arc::clone(&object.state);
            let record = Arc::clone(&object.service_record);
            let offset = Arc::clone(&object.offset);
            move || {
                run_disk(requests, SecondaryStorage::new(size), state, record, algorithm, offset);
            }
        });
        object
    }
    pub fn pause(&self) {
        self.state.store(0, Ordering::SeqCst);
    }
    pub fn run(&self) {
        self.state.store(1, Ordering::SeqCst);
    }
    pub fn shutdown(&self) {
        self.state.store(2, Ordering::SeqCst);
    }
    /// This is the sequential offset pointer, this is the pointer that is updated
    /// when we perform store operations.
    pub fn get_offset(&self) -> usize {
        self.offset.load(Ordering::SeqCst)
    }
}

fn run_disk(
    request_queue: Arc<IpcChannel<ServiceRequest>>,
    mut storage: SecondaryStorage,
    state: Arc<AtomicU8>,
    record: Arc<Mutex<Vec<usize>>>,
    algorithm: DiskAlgorithm,
    disk_offset: Arc<AtomicUsize>
) {
    // let algorithm = DiskAlgorithm::SSTF;

    let mut head = 0;
    let mut scan_forward = true;

    let mut clock = 0;
    let mut service_queue = vec![];

    loop {
        if state.load(Ordering::SeqCst) != 1 {
            yield_now();
            continue;
        }
        while let Some(item) = request_queue.try_recv() {
            let offset = match &item {
                ServiceRequest::Edit { addr, .. }
                | ServiceRequest::Read { addr, .. }
                | ServiceRequest::ReadBit { addr, .. }
                | ServiceRequest::WriteBit { addr, .. } => *addr,
                ServiceRequest::Write { .. } => RawStoragePtr {
                    byte_offset: storage.get_offset(),
                    bit_offset: 0,
                },
            };
            service_queue.push((offset, clock, item));
            clock += 1;
        }

        if algorithm == DiskAlgorithm::FCFS && !service_queue.is_empty() {
            // We are using first come first service and we also have an empty
            // service queue. We just get the first ones to come in.
            if let Some((index, _)) = service_queue
                .iter_mut()
                .enumerate()
                .min_by_key(|(_, (_, time, _))| *time)
            {
                let (offset, _, item) = service_queue.remove(index);
                service_request(item, offset, &mut storage, &record, &mut head, &disk_offset);
            }
        } else if algorithm == DiskAlgorithm::SSTF && !service_queue.is_empty() {
            // We are using shortest seek time first and thus we will choose
            // the request with the least distance.
            if let Some((index, _)) = service_queue
                .iter_mut()
                .enumerate()
                .min_by_key(|(_, (offset, _, _))| offset.byte_offset.abs_diff(head))
            {
                let (offset, _, item) = service_queue.remove(index);
                service_request(item, offset, &mut storage, &record, &mut head, &disk_offset);
            }
        } else if (algorithm == DiskAlgorithm::SCAN || algorithm == DiskAlgorithm::CSCAN || algorithm == DiskAlgorithm::CLOOK) && !service_queue.is_empty() {
            // We are using SCAN or CSCAN and thus we just service if we are on that spot.
            if let Some((index, _)) = service_queue
                .iter_mut()
                .enumerate()
                .find(|(_, (offset, _, _))| offset.byte_offset == head)
            {
                let (offset, _, item) = service_queue.remove(index);
                service_request(item, offset, &mut storage, &record, &mut head, &disk_offset);
            }
        }

        // This moves the head along if we are using scan or cscan.
        if algorithm == DiskAlgorithm::SCAN || algorithm == DiskAlgorithm::CSCAN || algorithm == DiskAlgorithm::CLOOK {
            head = if scan_forward { head + 1 } else { head - 1 };

            // If we are using CLOOK and there are no more requests in this direction jump o the beginning.
            if algorithm == DiskAlgorithm::CLOOK
                && service_queue.iter().find(|(o, _, _)| o.byte_offset >= head).is_none()
             {
                head = 0;
            } else if head >= storage.buffer.len() {
                if algorithm == DiskAlgorithm::SCAN {
                    scan_forward = false;
                } else {
                    head = 0;
                }
            } else if head <= 0 && !scan_forward {
                scan_forward = true;
            }
        }
    }
}

fn service_request(
    item: ServiceRequest,
    offset: RawStoragePtr,
    storage: &mut SecondaryStorage,
    record: &Mutex<Vec<usize>>,
    head: &mut usize,
    offset_disk: &AtomicUsize
) {
    record.lock().push(offset.byte_offset);
    *head = offset.byte_offset;
    match item {
        ServiceRequest::Read {
            addr,
            outbound,
            length,
        } => {
            outbound.send(storage.read(addr, length).to_vec());
        }
        ServiceRequest::Edit {
            addr,
            data,
            confirm,
        } => {
            storage.write(addr, &data);
            confirm.send(());
        }
        ServiceRequest::Write { bytes, inbound } => {
            inbound.send(storage.store(&bytes));
        }
        ServiceRequest::ReadBit { addr, outbound } => {
            outbound.send(storage.read_bit(addr));
        }
        ServiceRequest::WriteBit {
            addr,
            value,
            confirm,
        } => {
            storage.write_bit(addr, value);
            confirm.send(());
        }
    }
    offset_disk.store(storage.get_offset(), Ordering::SeqCst);
}

impl AbstractStorageDevice for MagneticDisk {
    fn write(&self, addr: super::RawStoragePtr, data: &[u8]) -> Yield<()> {
        let chan = Arc::new(IpcChannel::new());
        let request = ServiceRequest::Edit {
            addr,
            data: data.to_vec(),
            confirm: chan.clone(),
        };
        self.requests.send(request);
        Yield::new(chan)
    }
    fn read(&self, addr: super::RawStoragePtr, length: usize) -> Yield<Vec<u8>> {
        let chan = Arc::new(IpcChannel::new());
        let request = ServiceRequest::Read {
            addr,
            outbound: chan.clone(),
            length,
        };
        self.requests.send(request);
        Yield::new(chan)
    }
    fn store(&self, data: &[u8]) -> Yield<super::RawStoragePtr> {
        let chan = Arc::new(IpcChannel::new());
        let request = ServiceRequest::Write {
            bytes: data.to_vec(),
            inbound: chan.clone(),
        };
        self.requests.send(request);
        Yield::new(chan)
    }
    fn read_bit(&self, addr: RawStoragePtr) -> Yield<Bit> {
        let chan = Arc::new(IpcChannel::new());
        let request = ServiceRequest::ReadBit {
            addr,
            outbound: chan.clone(),
        };
        self.requests.send(request);
        Yield::new(chan)
    }
    fn write_bit(&self, addr: RawStoragePtr, value: Bit) -> Yield<()> {
        let chan = Arc::new(IpcChannel::new());
        let request = ServiceRequest::WriteBit {
            addr,
            value,
            confirm: chan.clone(),
        };
        self.requests.send(request);
        Yield::new(chan)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::disks::{hard_drive::DiskAlgorithm, AbstractStorageDevice, RawStoragePtr};

    use super::MagneticDisk;

    #[test]
    pub fn test_magnetic_disk_simple() {
        let magn = Arc::new(MagneticDisk::new(4096, DiskAlgorithm::FCFS));

        let wow = magn.store(&[1,2,3]).get();
        assert_eq!(magn.get_offset(), 3);
        assert_eq!(magn.read(wow, 3).get(), [1,2,3]);

        magn.write(RawStoragePtr::byte_ptr(50), &[4,5,6]).get();
        magn.write(RawStoragePtr::byte_ptr(25), &[4,5,6]).get();
    }

    #[test]
    pub fn test_magnetic_disk_servicing_fcfs() {
        let magn = Arc::new(MagneticDisk::new(4096, DiskAlgorithm::FCFS));
        magn.pause();

        let r1 = magn.store(&[1, 2, 3]);
        let r2 = magn.store(&[4, 5, 6]);
        let r3 = magn.write(RawStoragePtr::byte_ptr(96), &[7, 8]);
        let r4 = magn.write(RawStoragePtr::byte_ptr(50), &[7, 8]);

        magn.run();

        r1.get();
        r2.get();
        r3.get();
        r4.get();

        assert_eq!(*magn.service_record.lock(), [0, 0, 96, 50]);
    }


    #[test]
    pub fn test_magnetic_disk_servicing_sstf() {
        let magn = Arc::new(MagneticDisk::new(4096, DiskAlgorithm::SSTF));
        magn.pause();

        let r1 = magn.store(&[1, 2, 3]);
        let r2 = magn.store(&[4, 5, 6]);
        let r3 = magn.write(RawStoragePtr::byte_ptr(96), &[7, 8]);
        let r4 = magn.write(RawStoragePtr::byte_ptr(50), &[7, 8]);
        let r5 = magn.write(RawStoragePtr::byte_ptr(45), &[7, 8]);
        let r6 = magn.write(RawStoragePtr::byte_ptr(51), &[7, 8]);

        magn.run();

        r1.get();
        r2.get();
        r3.get();
        r4.get();
        r5.get();
        r6.get();

        assert_eq!(*magn.service_record.lock(), [0, 0, 45, 50, 51, 96]);
    }

    #[test]
    pub fn test_magnetic_disk_servicing_scan() {
        let magn = Arc::new(MagneticDisk::new(4096, DiskAlgorithm::SCAN));
        magn.pause();

        let r1 = magn.store(&[1, 2, 3]);
        let r2 = magn.store(&[4, 5, 6]);
        let r3 = magn.write(RawStoragePtr::byte_ptr(96), &[7, 8]);
        let r4 = magn.write(RawStoragePtr::byte_ptr(50), &[7, 8]);

        magn.run();

        r1.get();
        r2.get();
        r3.get();
        r4.get();

        assert_eq!(*magn.service_record.lock(), [0, 50, 96, 0]);
    }

    #[test]
    pub fn test_magnetic_disk_servicing_cscan() {
        let magn = Arc::new(MagneticDisk::new(4096, DiskAlgorithm::CSCAN));
        magn.pause();

        let r1 = magn.store(&[1, 2, 3]);
        let r2 = magn.store(&[4, 5, 6]);
        let r3 = magn.write(RawStoragePtr::byte_ptr(96), &[7, 8]);
        let r4 = magn.write(RawStoragePtr::byte_ptr(50), &[7, 8]);

        magn.run();

        r1.get();
        r2.get();
        r3.get();
        r4.get();

        assert_eq!(*magn.service_record.lock(), [0, 50, 96, 0]);
    }

    #[test]
    pub fn test_magnetic_disk_servicing_clook() {
        let magn = Arc::new(MagneticDisk::new(4096, DiskAlgorithm::CLOOK));
        magn.pause();

        let r1 = magn.store(&[1, 2, 3]);
        let r2 = magn.store(&[4, 5, 6]);
        let r3 = magn.write(RawStoragePtr::byte_ptr(96), &[7, 8]);
        let r4 = magn.write(RawStoragePtr::byte_ptr(50), &[7, 8]);

        magn.run();

        r1.get();
        r2.get();
        r3.get();
        r4.get();

        assert_eq!(*magn.service_record.lock(), [0, 50, 96, 0]);
    }

}
