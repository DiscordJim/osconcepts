use std::{thread::sleep, time::Duration};

use rand::{thread_rng, Rng};

pub mod memory;
pub mod computer;
pub mod disks;

pub struct Delay;

impl Delay {
    pub fn delay_random(millis: u64) {
        sleep(Duration::from_millis(thread_rng().gen_range(0..millis)));
    }
}