//! Asymmetric Multi-processing


use osconcepts::{computer::processor::Cpu, memory::pool::{MemoryMutex, RandomAccessMemory, SyncMemoryPtr}};



#[derive(Debug, PartialEq)]
pub enum Identity {
    Master,
    Slave
}


#[derive(Clone, Debug)]
pub struct CommonData {
    name: String
}

#[derive(Clone)]
pub struct MasterData {
    master: SyncMemoryPtr<MasterData>,
    common: SyncMemoryPtr<CommonData>
}

#[derive(Debug)]
pub struct CpuData {
    id: u8,
    identify: Identity,
    part: SyncMemoryPtr<CommonData>
}


/// Main kernel processor thread.
pub fn main_kernel(master: SyncMemoryPtr<MasterData>, common: SyncMemoryPtr<CommonData>) {
    println!("main launched");

}

/// Slave processor.
pub fn slave_processor(data: CpuData) {
    println!("slave launched");
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
    
}


pub fn main() {
    // each processor shares the same memory.
    let central_ram = RandomAccessMemory::<MemoryMutex>::new();
    // in this case our shared memory is a string.


    let shared_string = central_ram.store(CommonData {
        name: "Super Cool Computa".to_string()
    });

    let master_memory = central_ram.store(MasterData {
        common: shared_string.clone()
    });



    let slaves = vec![
        Cpu::new(CpuData {
            id: 1,
            identify: Identity::Slave,
            part: shared_string.clone()
        }),
        Cpu::new(CpuData {
            id: 2,
            identify: Identity::Slave,
            part: shared_string.clone()
        }),
        Cpu::new(CpuData {
            id: 3,
            identify: Identity::Slave,
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
            
        }
    }))

    for processor in slaves {
        joins.push(std::thread::spawn(move || {
            slave_processor(processor.data())
        }));
    }

    for join in joins {
        join.join().unwrap();
    }


}