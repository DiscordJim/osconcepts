//! Asymmetric Multi-processing
//! 
//! The kernel is on one CPU and the rest of the cores
//! are just slaves. The user thread communicates with the
//! master who schedule on other cores.


use std::{sync::Arc, thread, time::Duration};

use osconcepts::{computer::{process::{OpCode, Process}, processor::Cpu}, memory::{ipc::IpcChannel, pool::{MemoryMutex, RandomAccessMemory, SyncMemoryPtr}}};





#[derive(Clone)]
pub struct CommonData {
    tasks: Arc<IpcChannel<Process>>
}


pub struct MasterData {
    tasks: Arc<IpcChannel<Process>>
}


pub struct CpuData {
    id: u8,
    part: SyncMemoryPtr<CommonData>
}


/// Main kernel processor thread.
pub fn main_kernel(master: SyncMemoryPtr<MasterData>, common: SyncMemoryPtr<CommonData>) {
    println!("main launched");
    let channel = master.lock().get().tasks.clone();
    loop {
        let msg = channel.recv();
        println!("Master received process: {:?}", msg);
        if msg.code == OpCode::Shutdown {
            println!("Master received notice to shutdown, shutting down the slave cores.");
            for _ in 0..4 {
                common.lock().get().tasks.send(Process::shutdown());
            }
            break;
        } else {
            common.lock().get().tasks.send(msg);
        }
        
    
    }


}

/// Slave processor.
pub fn slave_processor(data: CpuData) {
    println!("slave launched");
    let channel = data.part.lock().get().tasks.clone();

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



pub fn user_thread(master: SyncMemoryPtr<MasterData>) {
    println!("launched the user thread");
    master.lock().get().tasks.send(Process::basic(0));
    master.lock().get().tasks.send(Process::basic(1));
    master.lock().get().tasks.send(Process::basic(2));
    master.lock().get().tasks.send(Process::basic(3));
    master.lock().get().tasks.send(Process::basic(4));

    // Shut down the master.
    master.lock().get().tasks.send(Process::shutdown());

}


pub fn main() {
    // each processor shares the same memory.
    let central_ram = RandomAccessMemory::<MemoryMutex>::new();
    // in this case our shared memory is a string.


    // Common Shared Memory
    let shared_string = central_ram.store(CommonData {
        tasks: Arc::new(IpcChannel::new())
    });

    // Master Memory
    let master_memory = central_ram.store(MasterData {
        tasks: IpcChannel::new().into(),
    });


    // Spawn the slave processors.
    let slaves = vec![
        Cpu::new(CpuData {
            id: 1,
            part: shared_string.clone()
        }),
        Cpu::new(CpuData {
            id: 2,
            part: shared_string.clone()
        }),
        Cpu::new(CpuData {
            id: 3,
            part: shared_string.clone()
        }),
        
    ];

    let mut joins = vec![];
    // startup the maser
    joins.push(std::thread::spawn({
        let master = master_memory.clone();
        let common = shared_string.clone();
        move || {
            main_kernel(master, common);
        }
    }));

    // statup the user
    joins.push(std::thread::spawn({
        let master = master_memory.clone();
        move || {
            user_thread(master);
        }
    }));

    for processor in slaves {
        joins.push(std::thread::spawn(move || {
            slave_processor(processor.data())
        }));
    }

    for join in joins {
        join.join().unwrap();
    }


}