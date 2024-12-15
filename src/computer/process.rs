use rand::{random, thread_rng};


#[derive(Debug)]
pub struct Process
{
    pub id: u32,
    /// How long the process will take.
    pub time_units: usize,
    pub code: OpCode,
    /// In actual operating systems this tends to be a mask
    pub affinity: i32
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
            code: OpCode::Inert,
            affinity: -1
        }
    }
    pub fn new(time: usize) -> Self {
        Self {
            id: random(),
            time_units: time,
            code: OpCode::Inert,
            affinity: -1
        }
    }
    pub fn full(id: u32, time: usize, code: OpCode) -> Self {
        Self {
            id,
            time_units: time,
            code,
            affinity: -1
        }
    }
    pub fn shutdown() -> Self {
        Self {
            id: random(),
            time_units: 0,
            code: OpCode::Shutdown,
            affinity: -1
        }
    }
    pub fn with_affinity(mut self, affinity: u32) -> Self {
        self.affinity = affinity as i32;
        self
    }
}

