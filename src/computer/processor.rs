use std::sync::Arc;


pub struct Cpu<D>(D);



impl<D> Cpu<D> {
    pub fn new(data: D) -> Self {
        Self(data)
    }
    pub fn data(self) -> D {
        self.0
    }
   
}