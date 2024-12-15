//! Symmetric Multi-processing
//! 
//! Each processor is self-scheduling.


use std::{sync::Arc, thread, time::Duration};

use osconcepts::{computer::{process::{OpCode, Process}, processor::Cpu}, memory::{ipc::IpcChannel, pool::{MemoryMutex, RandomAccessMemory, SyncMemoryPtr}}};





#[derive(Clone)]
pub struct CommonData {
    tasks: Arc<IpcChannel<Process>>
}




pub struct CpuData {
    /// The processor ID.
    id: u8,
    /// Common memory.
    shared: SyncMemoryPtr<CommonData>
}




/// Slave processor.
pub fn processor_core(data: CpuData) {
    println!("slave launched");
    let channel = data.shared.lock().get().tasks.clone();

    loop {
        let msg = channel.recv();
        println!("Process ({}) received work: {:?}", data.id, msg);



        if msg.code == OpCode::Shutdown {
            println!("Process ({}) received word to shut down.", data.id);
            break;
        } else {
            thread::sleep(Duration::from_secs(2));
        }
        
    }

}



pub fn user_thread(master: SyncMemoryPtr<CommonData>) {
    println!("launched the user thread");
    master.lock().get().tasks.send(Process::basic(0));
    master.lock().get().tasks.send(Process::basic(1));
    master.lock().get().tasks.send(Process::basic(2));
    master.lock().get().tasks.send(Process::basic(3));
    master.lock().get().tasks.send(Process::basic(4));

    // Shut down the master.
    for _ in 0..3 {
        master.lock().get().tasks.send(Process::shutdown());
    }

}


pub fn main() {
    // each processor shares the same memory.
    let central_ram = RandomAccessMemory::<MemoryMutex>::new();
    // in this case our shared memory is a string.


    // Common Shared Memory
    let shared_string = central_ram.store(CommonData {
        tasks: Arc::new(IpcChannel::new())
    });


    // Spawn the slave processors.
    let slaves = vec![
        Cpu::new(CpuData {
            id: 1,
            shared: shared_string.clone()
        }),
        Cpu::new(CpuData {
            id: 2,
            shared: shared_string.clone()
        }),
        Cpu::new(CpuData {
            id: 3,
            shared: shared_string.clone()
        }),
        
    ];

    let mut joins = vec![];

    // statup the user
    joins.push(std::thread::spawn({
        let master = shared_string.clone();
        move || {
            user_thread(master);
        }
    }));

    for processor in slaves {
        joins.push(std::thread::spawn(move || {
            processor_core(processor.data())
        }));
    }

    for join in joins {
        join.join().unwrap();
    }


}