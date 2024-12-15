use osconcepts::{computer::processor::Cpu, memory::pool::{MemoryPtr, RandomAccessMemory}};

/// Uniform Memory Access

#[derive(Debug)]
pub struct CpuData {
    id: u8,
    part: MemoryPtr<String>
}


pub fn cpu_function(mut data: CpuData) {
    for _ in 0..50 {
        let buffer = if data.id == 0 {
            random_string::generate(6, "ABCabc")
        } else {
            random_string::generate(6, "DEFdef")
        };
        *data.part = buffer;
    }
    
}

pub fn main() {
    // each processor shares the same memory.
    let central_ram = RandomAccessMemory::new();
    // in this case our shared memory is a string.
    let shared_string = central_ram.store("hello".to_string());


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