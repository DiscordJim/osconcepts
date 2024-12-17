use std::{fmt::Debug, ops::Index, slice::SliceIndex};

pub type Bit = bool;

/// A bit vector that stores the bit values as booleans, this is
/// for working with bits in an easy way.
#[derive(Debug, Clone)]
pub struct BitVec([bool; 8]);

impl BitVec {
    /// Calculates the parity of the bit vector by counting the ones
    /// and informing us if they are odd.
    pub fn parity(&self) -> Bit {
        let mut ones = 0;
        for i in 0..self.0.len() {
            if self.0[i] {
                ones += 1;
            }
        }
        ones % 2 == 1
    }
}


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

    #[test]
    pub fn simple_bitvec_parity() {
        assert_eq!(BitVec::from(3 as u8).parity(), false);
        assert_eq!(BitVec::from(1 as u8).parity(), true);
    }
}