use std::{fmt::Debug, ops::Index, slice::SliceIndex};

pub type Bit = bool;


#[derive(Debug)]
pub struct BitVec([bool; 8]);


impl From<u8> for BitVec {
    fn from(value: u8) -> Self {
        let mut vec = [false; 8];
        for i in 0..8 {
            vec[i] = ((value >> (7 - i)) & 1) != 0;
        }
        Self(vec)
    }
}

impl<Idx: SliceIndex<[bool]>> Index<Idx> for BitVec {
    type Output = <Idx as SliceIndex<[bool]>>::Output;
    fn index(&self, index: Idx) -> &Self::Output {
        &self.0[index]
    }
}



impl From<BitVec> for u8 {
    fn from(value: BitVec) -> Self {
        let mut number = 0;
        for i in 0..8 {
            if value.0[i] {
                number |= 1 << (7 - i);
            }
        }
        number
    }
}


#[cfg(test)]
mod tests {
    use crate::disks::bits::BitVec;


    #[test]
    pub fn simple_bitvec() {
        for i in 0..u8::MAX {
            let bv = BitVec::from(i);
            let bv: u8 = bv.into();
            assert_eq!(bv, i);
        }
    }
}