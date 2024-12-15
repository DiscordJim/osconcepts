
#[derive(Debug)]
pub struct Process {
    id: u32
}

impl Process {
    pub fn new(id: u32) -> Self {
        Self {
            id
        }
    }
}

impl Process {}