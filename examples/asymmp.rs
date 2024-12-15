//! Asymmetric Multi-processing


use std::{collections::VecDeque, sync::Arc, thread, time::Duration};

use osconcepts::{computer::{process::Process, processor::Cpu}, memory::{ipc::IpcChannel, pool::{MemoryMutex, RandomAccessMemory, SyncMemoryPtr}}};
use parking_lot::{Condvar, Mutex};





#[derive(Clone)]
pub struct CommonData {
    name: String,
    tasks: Arc<IpcChannel<String>>
}


pub struct MasterData {
    tasks: Arc<IpcChannel<String>>,
    common: SyncMemoryPtr<CommonData>
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
        println!("Received a message, {}", msg);
        common.lock().get().tasks.send(msg);
    
    }


}

/// Slave processor.
pub fn slave_processor(data: CpuData) {
    println!("slave launched");
    let channel = data.part.lock().get().tasks.clone();

    loop {
        let msg = channel.recv();
        println!("Process ({}) received work: {}", data.id, msg);
        thread::sleep(Duration::from_secs(2));
    }

}

// pub fn cpu_function(data: CpuData) {
//     match data.identify {
//         Identity::Master => main_kernel(data),
//         Identity::Slave => slave_processor(data),
//     }
//     // for _ in 0..50 {
//     //     let buffer = if data.id == 0 {
//     //         random_string::generate(6, "ABCabc")
//     //     } else {
//     //         random_string::generate(6, "DEFdef")
//     //     };
//     //     *data.part.lock().get_mut() = buffer;
//     // }
// }

pub fn user_thread(master: SyncMemoryPtr<MasterData>) {
    println!("launched the user thread");
    master.lock().get().tasks.send("Goodmorning masta!".to_string());
    master.lock().get().tasks.send("good afternoon masta!".to_string());
    master.lock().get().tasks.send("goodnight masta!".to_string());
    master.lock().get().tasks.send("Goodmorning masta! 2".to_string());
    master.lock().get().tasks.send("good afternoon masta! 2".to_string());
    master.lock().get().tasks.send("goodnight masta! 2".to_string());
}


pub fn main() {
    // each processor shares the same memory.
    let central_ram = RandomAccessMemory::<MemoryMutex>::new();
    // in this case our shared memory is a string.


    let shared_string = central_ram.store(CommonData {
        name: "Super Cool Computa".to_string(),
        tasks: Arc::new(IpcChannel::new())
    });

    let master_memory = central_ram.store(MasterData {
        tasks: IpcChannel::new().into(),
        common: shared_string.clone()
    });


    // spawn all the slavs
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