//! Asymmetric Multi-processing with Dynamic Dispatching
//! 
//! The kernel is on one CPU and the rest of the cores
//! are just slaves. The user thread communicates with the
//! master who schedule on other cores.
//! 
//! To keep it simple when the time quantum is elapsed it is 
//! just put back into the master quuee.


use std::{sync::Arc, thread, time::Duration};

use osconcepts::{computer::{process::{OpCode, Process}, processor::Cpu}, memory::{ipc::IpcChannel, pool::{MemoryMutex, RandomAccessMemory, SyncMemoryPtr}}};

const TIME_UNIT_DURATION: Duration = Duration::from_millis(50);

/// How long a task should run for (in time units) until it is
/// rescheduled?
const QUANTA: usize = 3;


#[derive(Clone)]
pub struct CommonData {
    /// The queue for the master processors.
    master_queue: Arc<IpcChannel<Process>>,
    /// The queue for the slave processors.
    slave_queue: Arc<IpcChannel<Process>>
}

/// The data for the processor.
pub struct CpuData {
    id: u8,
    part: SyncMemoryPtr<CommonData>
}


/// Main kernel processor thread.
pub fn main_kernel(common: SyncMemoryPtr<CommonData>) {
    println!("main launched");
    let channel = common.lock().get().master_queue.clone();
    loop {
        let msg = channel.recv();
        println!("Master received process: {:?}", msg);
        if msg.code == OpCode::Shutdown && msg.time_units == 0 {
            println!("Master received notice to shutdown, shutting down the slave cores.");
            for _ in 0..4 {
                common.lock().get().slave_queue.send(Process::shutdown());
            }
            break;
        } else {
            common.lock().get().slave_queue.send(msg);
        }
        
    
    }


}

/// Slave processor.
pub fn slave_processor(data: CpuData) {
    println!("slave launched");

    let master_work_queue = data.part.lock().get().master_queue.clone();
    let slave_work_queue = data.part.lock().get().slave_queue.clone();

    // Allows us to time the task on the CPU. 
    let mut clock = QUANTA;
    loop {
        let mut msg = slave_work_queue.recv();
        println!("Processor ({}) received work: {:?}", data.id, msg);

        if msg.code == OpCode::Shutdown && msg.time_units == 0 {
            println!("Processor ({}) received word to shut down.", data.id);
            break;
        } else {
            // Tick the time quantum.
            while msg.time_units > 0 && clock > 0 {
                thread::sleep(TIME_UNIT_DURATION);
                msg.time_units -= 1;
                clock -= 1;
            }
            println!("Processor ({}) elapsed the time quantum for the process {:?}.", data.id, msg);
            clock = QUANTA;

            // Reschedule it back into the primary queue.
            // There is a special case where it is a shutdown,
            // shutdowns need to be boradcasted.
            if msg.time_units > 0 || (msg.code == OpCode::Shutdown) {
                master_work_queue.send(msg);
            }

        }
        
    }

}



pub fn user_thread(master: SyncMemoryPtr<CommonData>) {
    println!("launched the user thread");

    // We will first launch three processes.
    master.lock().get().slave_queue.send(Process::new(10));
    master.lock().get().slave_queue.send(Process::new(15));
    master.lock().get().slave_queue.send(Process::new(5));


    // We then wait for a certain amount of time.
    std::thread::sleep(Duration::from_secs(2));
    println!("User is shutting down the processor.");


    // Now we will send a timed shutdown signal. 
    master.lock().get().slave_queue.send(Process::full(10, OpCode::Shutdown));

}


pub fn main() {
    // each processor shares the same memory.
    let central_ram = RandomAccessMemory::<MemoryMutex>::new();
    // in this case our shared memory is a string.


    // Common Shared Memory
    let shared_string = central_ram.store(CommonData {
        master_queue: Arc::new(IpcChannel::new()),
        slave_queue: Arc::new(IpcChannel::new())
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
        let master = shared_string.clone();
        move || {
            main_kernel(master);
        }
    }));

    // statup the user
    joins.push(std::thread::spawn({
        let master = shared_string.clone();
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