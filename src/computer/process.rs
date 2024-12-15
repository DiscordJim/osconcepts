use rand::{random, thread_rng};


#[derive(Debug)]
pub struct Process
{
    id: u32,
    pub code: OpCode
}

#[derive(Debug, PartialEq)]
pub enum OpCode {
    Shutdown,
    Inert
}

impl Process
{
    pub fn basic(id: u32) -> Self {
        Self {
            id,
            code: OpCode::Inert
        }
    }
    pub fn shutdown() -> Self {
        Self {
            id: random(),
            code: OpCode::Shutdown
        }
    }
}

