//! Implementation of a basic CPU scheduler.

use std::{
    collections::{HashMap, VecDeque}, ops::{Deref, DerefMut}
};

use super::process::Process;

const INITIAL_TAU: f32 = 10.0;

#[derive(Debug, PartialEq)]
pub enum SchedulerAlgorithm {
    /// First come first serve algorithm.
    FirstComeFirstServe,
    /// Non-preemptive priority scheduling algorithm.
    Priority,
    /// Preemptive priority scheduling algorithm,
    /// if a process comes in with higher priority it
    /// will knock off the currently scheduled one.
    PreemptivePriority,
    /// Round robin scheduling algorithm, each
    /// process runs for a certain time quantum.
    RoundRobin(usize),
    /// This is a preemptive scheduling algorithm
    /// that schedules the shortest job next.
    ShortestRemainingTime(f32),
}

#[derive(Debug)]
pub struct ProcessRecord {
    /// The time the process was scheduled.
    schedule_time: u128,

    /// This is the lifetime of the process
    /// under round robin.
    lifetime: i32,

    /// Estimated remaining time, this is for SRT.
    estimated_remaining_time: f32,

    /// The actual process.
    pub proc: Process,
}

impl ProcessRecord {
    pub fn tick(&mut self) {
        if self.lifetime > 0 {
            self.lifetime -= 1;
        }
        if self.proc.time_units > 0 {
            self.proc.time_units -= 1;
        }
        self.estimated_remaining_time -= 1.0;
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

pub struct Normal;
pub struct Feedback;

pub struct Scheduler {
    /// The currently scheduled process.
    scheduled: Option<ProcessRecord>,

    /// The queue of currently scheduled
    /// processes.
    queue: VecDeque<ProcessRecord>,

    /// Is feedback queue?
    feedback: bool,

    /// The scheduling algorithm to be
    /// used.
    policy: SchedulerAlgorithm,

    /// This is the insertion clock, it gets
    /// incremmented every time we add something
    /// to the queue.
    clock: u128,

    /// For the shortest time remaining algorithm,
    /// will be intialized to an initial value and updated
    /// upon getting more information during runs.
    srt_time_table: HashMap<u32, f32>,


}



impl Scheduler {
    pub fn new(policy: SchedulerAlgorithm) -> Self {
        Self {
            policy,
            scheduled: None,
            queue: VecDeque::default(),
            srt_time_table: HashMap::new(),
            feedback: false,
            clock: 0,

        }
    }
    pub fn with_feedback(mut self) -> Self {
        self.feedback = true;
        self
    }
    /// Schedules a new process onto the scheduler.
    /// 
    /// If this is in feedback mode, whenever something
    /// gets preempted or moved off it will be bumped off
    /// and returned by this function.
    pub fn schedule(&mut self, process: Process) -> Option<ProcessRecord> {
        self.schedule_inner(ProcessRecord {
            schedule_time: self.clock,
            lifetime: 0,
            estimated_remaining_time: INITIAL_TAU,
            proc: process,
        })
    }
    /// Schedules a process record onto the scheduler.
    /// 
    /// If this is in feedback mode, whenever something
    /// gets preempted or moved off it will be bumped off
    /// and returned by this function.
    fn schedule_inner(&mut self, mut record: ProcessRecord) -> Option<ProcessRecord> {
        if !self.srt_time_table.contains_key(&record.id) {
            // If this is not in the table, store the default value.
            self.srt_time_table.insert(record.id, INITIAL_TAU);
        }

        record.schedule_time = self.clock;

        if self.scheduled.is_none() {
            // No current scheduled task, so just schedule it directly.
            self.set_scheduled_record(record);
        } else if self.policy == SchedulerAlgorithm::PreemptivePriority
            && self.scheduled.is_some()
            && record.proc.priority < self.scheduled.as_ref().unwrap().proc.priority
        {
            let current=  self.scheduled.take();
           
            
            self.set_scheduled_record(record);

            if !self.feedback {
                // If we are not in feedback mode, 
                // we should push this back into the queue..
                self.queue.push_back(current.unwrap());
            } else {
                // Kick the value back since we are in feedback mode.
                return current;
            }

        } else if matches!(self.policy, SchedulerAlgorithm::ShortestRemainingTime(_))
            && self.scheduled.is_some()
            // Check if the incoming process has a shorter time than the current.
            && self.scheduled.as_ref().unwrap().estimated_remaining_time > *self.srt_time_table.get(&record.id).unwrap()
        {
            let current = self.scheduled.take();
            
            self.set_scheduled_record(record);

            if !self.feedback {
                // If we are not in feedback mode push back into queue.
                self.queue.push_back(current.unwrap());
            } else {
                // Kick the value back since we are in feedback mode.
                return current;
            }
        } else {
            self.queue.push_back(record);
            self.clock += 1;
        }
        None

        
    }
    fn set_scheduled(&mut self, record: Option<ProcessRecord>) {
        match record {
            Some(record) => self.set_scheduled_record(record),
            None => self.scheduled = None,
        }
    }
    fn set_scheduled_record(&mut self, mut record: ProcessRecord) {
        // Set the estimated remaining time. This is for shortest time remaining.
        record.estimated_remaining_time = *self.srt_time_table.get(&record.id).unwrap();


        if let SchedulerAlgorithm::RoundRobin(quantum) = self.policy {
            record.lifetime = quantum.try_into().unwrap();
        }
        self.scheduled = Some(record);
    }
    pub fn current_unchecked(&mut self) -> &mut ProcessRecord {
        self.current().unwrap()
    }
    pub fn current(&mut self) -> Option<&mut ProcessRecord> {
        self.fetch_current().0
    }
    /// Fectches the current value and will return the old one if we
    /// are in round robin. This is for implementing multi-level feedback queues.
    pub fn fetch_current(&mut self) -> (Option<&mut ProcessRecord>, Option<ProcessRecord>) {
        let mut bumped = None;
        if self.scheduled.is_some() {
            if self.scheduled.as_ref().unwrap().proc.time_units == 0 {
                let next = self.next();

                // Update the shortest time remaining table.
                if let SchedulerAlgorithm::ShortestRemainingTime(alpha) = self.policy {
                    // Update the prediction.
                    let tau = self
                        .srt_time_table
                        .get_mut(&self.scheduled.as_ref().unwrap().id)
                        .unwrap();
                    *tau = (alpha * (self.scheduled.as_ref().unwrap().static_time_units as f32))
                        + ((1.0 - alpha) * (*tau));
                }
                self.set_scheduled(next);
            } else if matches!(self.policy, SchedulerAlgorithm::RoundRobin(_))
                && self.scheduled.as_ref().unwrap().lifetime <= 0
            {
                // We are using round robin and the time quantum has expired.
                let current = self.scheduled.take();
                let next = self.next();
                self.set_scheduled(next);
                if !self.feedback {
                    // If we are not in feedback mode, then we want to reschedule.
                    self.schedule_inner(current.unwrap());
                } else {
                    // We are in feedback mode, bump the process back.
                    bumped = current;
                }
                
            }
        }
        (self.scheduled.as_mut(), bumped)
    }
    fn next(&mut self) -> Option<ProcessRecord> {
        match self.policy {
            SchedulerAlgorithm::FirstComeFirstServe | SchedulerAlgorithm::RoundRobin(_) => {
                let (index, _) = self
                    .queue
                    .iter_mut()
                    .enumerate()
                    .min_by_key(|(_, f)| f.schedule_time)?;
                self.queue.remove(index)
            }
            SchedulerAlgorithm::Priority | SchedulerAlgorithm::PreemptivePriority => {
                let (index, _) = self
                    .queue
                    .iter_mut()
                    .enumerate()
                    .min_by_key(|(_, f)| f.proc.priority)?;
                self.queue.remove(index)
            }
            SchedulerAlgorithm::ShortestRemainingTime(_) => {
                // Note: tau is our estimated time remaining.
                // Therefore, we grab the process with the lowest remaining time.
                let (index, _) = self.queue.iter_mut().enumerate().min_by(|(_, a), (_, b)| {
                    a.estimated_remaining_time
                        .total_cmp(&b.estimated_remaining_time)
                })?;
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
        let mut scheduler = Scheduler::new(SchedulerAlgorithm::FirstComeFirstServe);
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
    pub fn scheduler_srt_preemption() {
        let mut scheduler = Scheduler::new(SchedulerAlgorithm::ShortestRemainingTime(0.5));
        scheduler.schedule(Process::full(0, 3, OpCode::Inert));

        // tick it to the end
        scheduler.current_unchecked().tick_n(3);

        // current should be one
        scheduler.schedule(Process::full(1, 3, OpCode::Inert));
        assert_eq!(scheduler.current_unchecked().id, 1);

        // this should preempt one.
        scheduler.schedule(Process::full(0, 3, OpCode::Inert));
        assert_eq!(scheduler.current_unchecked().id, 0);
    }

    #[test]
    pub fn scheduler_srt() {
        let mut scheduler = Scheduler::new(SchedulerAlgorithm::ShortestRemainingTime(0.5));
        scheduler.schedule(Process::full(0, 3, OpCode::Inert));

        // tick it to the end
        scheduler.current_unchecked().tick_n(3);

        // we should have nothing i queue.
        assert!(scheduler.current().is_none());

        // reschedule a thread from process 0
        scheduler.schedule(Process::full(0, 3, OpCode::Inert));

        // make sure the calculation is correct.
        assert!(scheduler.srt_time_table.get(&0).unwrap().eq(&6.5));
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
