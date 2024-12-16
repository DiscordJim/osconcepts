use super::hard_drive::MagneticDisk;


pub struct Raid0 {
    array: Vec<MagneticDisk>
}

impl Raid0 {
    pub fn new() -> Self {
        Self {
            array: vec![]
        }
    }
    pub fn with_disk(mut self, disk: MagneticDisk) -> Self {
        self.array.push(disk);
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::disks::hard_drive::MagneticDisk;

    use super::Raid0;

    // #[test]
    // pub fn test_raid0_array() {
    //     let raid = Raid0::new()
    //         .with_disk(MagneticDisk::new(256))
    //         .with_disk(MagneticDisk::new(256))
    //         .with_disk(MagneticDisk::new(256));


    // }
}