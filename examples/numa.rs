//! Non-Uniform Memory Access
//! 
//! The delays are largely provided by the [NumaPtr].
//! 
//! When [NumaPtr] is locked, it will have a slight random delay until
//! it can acquire a lock and then another slight random delay to fetch the value.
//! 
//! This provides the desired Non-Uniform Memory Access

use osconcepts::{computer::processor::Cpu, memory::{numa::NumaPtr, pool::{MemoryMutex, RandomAccessMemory}}};



#[derive(Debug)]
pub struct CpuData {
    id: u8,
    part: NumaPtr<String>
}





pub fn cpu_function(data: CpuData) {
    for _ in 0..50 {
        let buffer = if data.id == 0 {
            random_string::generate(6, "ABCabc")
        } else {
            random_string::generate(6, "DEFdef")
        };
        *data.part.lock().get_mut() = buffer;
    }
}

pub fn main() {
    // each processor shares the same memory.
    let central_ram = RandomAccessMemory::<MemoryMutex>::new();
    // in this case our shared memory is a string.
    let shared_string = NumaPtr::upgrade(central_ram.store("hello".to_string()));


    let processors = vec![
        Cpu::new(CpuData {
            id: 0,
            part: shared_string.clone()


        }),
        Cpu::new(CpuData {
            id: 1,
            part: shared_string.clone()
        })
    ];

    let mut joins = vec![];
    for processor in processors {
        joins.push(std::thread::spawn(move || {
            cpu_function(processor.data())
        }));
    }

    for join in joins {
        join.join().unwrap();
    }


}