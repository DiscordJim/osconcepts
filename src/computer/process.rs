use rand::{random, thread_rng};


#[derive(Debug)]
pub struct Process
{
    id: u32,
    /// How long the process will take.
    pub time_units: usize,
    pub code: OpCode
}

#[derive(Debug, PartialEq)]
pub enum OpCode {
    Shutdown,
    Inert
}

impl Process
{
    pub fn dummy(id: u32) -> Self {
        Self {
            id,
            time_units: 0,
            code: OpCode::Inert
        }
    }
    pub fn new(time: usize) -> Self {
        Self {
            id: random(),
            time_units: time,
            code: OpCode::Inert
        }
    }
    pub fn full(time: usize, code: OpCode) -> Self {
        Self {
            id: random(),
            time_units: time,
            code
        }
    }
    pub fn shutdown() -> Self {
        Self {
            id: random(),
            time_units: 0,
            code: OpCode::Shutdown
        }
    }
}

