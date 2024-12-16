//! Implementation of a basic CPU scheduler.

use std::{collections::{LinkedList, VecDeque}, ops::{Deref, DerefMut}, time::SystemTime};

use super::process::Process;


const INITIAL_TAU: f32 = 10.0;

#[derive(Debug, PartialEq)]
pub enum SchedulerAlgorithm {
    FCFS,
    Priority,
    PreemptivePriority,
    RoundRobin(usize)
}

pub struct ProcessRecord {
    insertion_time: u128,
    schedule_time: i128,
    lifetime: i32,
    /// This is the calculated estimated length.
    tau: f32,
    proc: Process
}

impl ProcessRecord {
    pub fn tick(&mut self) {
        if self.lifetime > 0 {
            self.lifetime -= 1;
        }
        if self.proc.time_units > 0 {
            self.proc.time_units -= 1;
        }
    }
    pub fn tick_n(&mut self, n: usize) {
        for _ in 0..n {
            self.tick();
        }
    }
}

impl Deref for ProcessRecord {
    type Target = Process;
    fn deref(&self) -> &Self::Target {
        &self.proc
    }
}

impl DerefMut for ProcessRecord {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.proc
    }
}

pub struct Scheduler {
    scheduled: Option<ProcessRecord>,
    queue: VecDeque<ProcessRecord>,
    policy: SchedulerAlgorithm,
    clock: u128
}

impl Scheduler {
    pub fn new(policy: SchedulerAlgorithm) -> Self {
        Self {
            policy,
            scheduled: None,
            queue: VecDeque::default(),
            clock: 0,
        }
    }
    pub fn schedule(&mut self, process: Process) {
        self.schedule_inner(ProcessRecord {
            insertion_time: self.clock,
            schedule_time: -1,
            lifetime: 0,
            tau: INITIAL_TAU,
            proc: process
        });
    }

    fn schedule_inner(&mut self, mut record: ProcessRecord) {
        

        record.schedule_time = -1;
        record.insertion_time = self.clock;
        if self.scheduled.is_none() {
            self.set_scheduled_record(record);
        } else if self.policy == SchedulerAlgorithm::PreemptivePriority
            && self.scheduled.is_some()
            && record.proc.priority < self.scheduled.as_ref().unwrap().proc.priority {
            self.queue.push_back(self.scheduled.take().unwrap());
            self.scheduled = Some(record);
        } else {
            self.queue.push_back(record);
            self.clock += 1;
        }
    }
    fn set_scheduled(&mut self, record: Option<ProcessRecord>) {
        match record {
            Some(record) => self.set_scheduled_record(record),
            None => self.scheduled = None
        }
    }
    fn set_scheduled_record(&mut self, mut record: ProcessRecord) {
        record.schedule_time = self.clock.try_into().unwrap();
        if let SchedulerAlgorithm::RoundRobin(quantum) = self.policy {
            record.lifetime = quantum.try_into().unwrap();
        }
        self.scheduled = Some(record);
    }
    pub fn current_unchecked(&mut self) -> &mut ProcessRecord {
        self.current().unwrap()
    }
    pub fn current(&mut self) -> Option<&mut ProcessRecord> {
        if self.scheduled.is_some() {
        
            if self.scheduled.as_ref().unwrap().proc.time_units == 0 {
                let next = self.next();
                self.set_scheduled(next);
            
            } else if matches!(self.policy, SchedulerAlgorithm::RoundRobin(_)) && self.scheduled.as_ref().unwrap().lifetime <= 0 {
                println!("This matches?");
                // We are using round robin and the time quantum has expired.
                let current = self.scheduled.take().unwrap();
                let next = self.next();
                self.set_scheduled(next);
                self.schedule_inner(current);
            }
        }
        self.scheduled.as_mut()
    }
    fn next(&mut self) -> Option<ProcessRecord> {
        match self.policy {
            SchedulerAlgorithm::FCFS | SchedulerAlgorithm::RoundRobin(_) => {
                let (index, _) = self.queue.iter_mut().enumerate().min_by_key(|(_, f)| f.insertion_time)?;
                self.queue.remove(index)
            },
            SchedulerAlgorithm::Priority | SchedulerAlgorithm::PreemptivePriority => {
                let (index, _) = self.queue.iter_mut().enumerate().min_by_key(|(_, f)| f.proc.priority)?;
                self.queue.remove(index)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::computer::process::{OpCode, Process};

    use super::{Scheduler, SchedulerAlgorithm};


    #[test]
    pub fn scheduler_fcfs() {
        let mut scheduler = Scheduler::new(SchedulerAlgorithm::FCFS);
        scheduler.schedule(Process::full(0, 1, OpCode::Inert));
        scheduler.schedule(Process::full(1, 1, OpCode::Inert));
        scheduler.schedule(Process::full(4, 1, OpCode::Inert));
        scheduler.schedule(Process::full(2, 1, OpCode::Inert));

        assert_eq!(scheduler.current_unchecked().proc.id, 0);
        scheduler.current_unchecked().proc.time_units = 0;
        assert_eq!(scheduler.current_unchecked().proc.id, 1);
        scheduler.current_unchecked().proc.time_units = 0;
        assert_eq!(scheduler.current_unchecked().proc.id, 4);
        scheduler.current_unchecked().proc.time_units = 0;
        assert_eq!(scheduler.current_unchecked().proc.id, 2);
    }

    #[test]
    pub fn scheduler_priority() {
        let mut scheduler = Scheduler::new(SchedulerAlgorithm::Priority);
        scheduler.schedule(Process::full(0, 1, OpCode::Inert).with_prioirty(5));
        scheduler.schedule(Process::full(1, 1, OpCode::Inert));
        scheduler.schedule(Process::full(4, 1, OpCode::Inert).with_prioirty(-20));
        scheduler.schedule(Process::full(2, 1, OpCode::Inert));

        assert_eq!(scheduler.current_unchecked().proc.id, 0);
        scheduler.current_unchecked().proc.time_units = 0;
        assert_eq!(scheduler.current_unchecked().proc.id, 4);
        scheduler.current_unchecked().proc.time_units = 0;
        assert_eq!(scheduler.current_unchecked().proc.id, 1);
        scheduler.current_unchecked().proc.time_units = 0;
        assert_eq!(scheduler.next().unwrap().proc.id, 2);
    }

    #[test]
    pub fn scheduler_priority_preempt() {
        let mut scheduler = Scheduler::new(SchedulerAlgorithm::PreemptivePriority);
        scheduler.schedule(Process::full(0, 1, OpCode::Inert).with_prioirty(5));
        scheduler.schedule(Process::full(1, 1, OpCode::Inert).with_prioirty(-1));
        scheduler.schedule(Process::full(4, 1, OpCode::Inert).with_prioirty(-20));
        scheduler.schedule(Process::full(2, 1, OpCode::Inert));

        assert_eq!(scheduler.current_unchecked().proc.id, 4);
        scheduler.current_unchecked().proc.time_units = 0;
        assert_eq!(scheduler.current_unchecked().proc.id, 1);
  

        scheduler.schedule(Process::full(5, 1, OpCode::Inert).with_prioirty(-50));
        assert_eq!(scheduler.current_unchecked().proc.id, 5);
        scheduler.current_unchecked().proc.time_units = 0;

        assert_eq!(scheduler.current_unchecked().proc.id, 1);
        scheduler.current_unchecked().proc.time_units = 0;

        assert_eq!(scheduler.current_unchecked().proc.id, 2);
        scheduler.current_unchecked().proc.time_units = 0;

        


    }

    #[test]
    pub fn scheduler_rr() {
        let mut scheduler = Scheduler::new(SchedulerAlgorithm::RoundRobin(3));
        scheduler.schedule(Process::full(0, 6, OpCode::Inert));
        scheduler.schedule(Process::full(1, 3, OpCode::Inert));

        // Tick the first out.
        assert_eq!(scheduler.current_unchecked().id, 0);
        scheduler.current_unchecked().tick();
        assert_eq!(scheduler.current_unchecked().id, 0);
        scheduler.current_unchecked().tick();
        scheduler.current_unchecked().tick();

        // The quantum of three should now be expired, schedule
        // process one.
        assert_eq!(scheduler.current_unchecked().id, 1);
        scheduler.current_unchecked().tick();
        scheduler.current_unchecked().tick();
        scheduler.current_unchecked().tick();


        // We should be back to process 0
        assert_eq!(scheduler.current_unchecked().id, 0);
        scheduler.current_unchecked().tick();
        scheduler.current_unchecked().tick();
        scheduler.current_unchecked().tick();

       
        // we are done
        assert!(scheduler.current().is_none());
       

    }
}