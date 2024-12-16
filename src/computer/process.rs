use rand::random;


pub enum ProcessState {
    New,
    Ready,
    Running,
    Blocked
}

#[derive(Debug)]
pub struct Process
{
    /// The process PID.
    pub id: u32,
    /// Process priority.
    pub priority: i32,
    /// How long the process will take.
    pub time_units: usize,
    /// Full time units
    pub static_time_units: usize,

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
            priority: 0,
            time_units: 0,
            static_time_units: 0,
            code: OpCode::Inert,
            affinity: -1
        }
    }
    pub fn new(time: usize) -> Self {
        Self {
            id: random(),
            priority: 0,
            time_units: time,
            static_time_units: time,
            code: OpCode::Inert,
            affinity: -1
        }
    }
    pub fn full(id: u32, time: usize, code: OpCode) -> Self {
        Self {
            id,
            priority: 0,
            static_time_units: time,
            time_units: time,
            code,
            affinity: -1
        }
    }
    pub fn shutdown() -> Self {
        Self {
            id: random(),
            priority: 0,
            time_units: 0,
            static_time_units: 0,
            code: OpCode::Shutdown,
            affinity: -1
        }
    }
    pub fn with_affinity(mut self, affinity: u32) -> Self {
        self.affinity = affinity as i32;
        self
    }
    pub fn with_prioirty(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

