use std::collections::VecDeque;

use super::{process::Process, scheduler::{ProcessRecord, Scheduler, SchedulerAlgorithm}};


/// A simple multilevel feedback queue.
/// 
/// This queue works by having multiple levels
/// and taking the current scheduled task from the highest
/// available queue. This is quite efficient and is a simple
/// way to implement it.
/// 
/// ```
/// use osconcepts::computer::multilevel::MultilevelQueue;
/// use osconcepts::computer::scheduler::SchedulerAlgorithm;
/// 
/// let mut queue = MultilevelQueue::new()
///     .with_level(SchedulerAlgorithm::RoundRobin(2))
///     .with_level(SchedulerAlgorithm::RoundRobin(4));
/// ```
#[derive(Default)]
pub struct MultilevelQueue {
    levels: VecDeque<Scheduler>
}

impl MultilevelQueue {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_level(mut self, level: SchedulerAlgorithm) -> Self {
        self.levels.push_back(Scheduler::new(level));
        self
    }
    pub fn schedule(&mut self, process: Process) {
        self.levels[0].schedule(process);
    }
    pub fn current_unchecked(&mut self) -> &mut ProcessRecord {
        self.current().unwrap()
    }
    pub fn current(&mut self) -> Option<&mut ProcessRecord> {
        for i in 0..self.levels.len() {
            if self.levels[i].current().is_some() {
                return self.levels[i].current();
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::computer::{process::{OpCode, Process}, scheduler::SchedulerAlgorithm};

    use super::MultilevelQueue;



    #[test]
    pub fn test_multilevel_simple() {
        // Form the multi-level feedback quuee.
        let mut queue = MultilevelQueue::new()
            .with_level(SchedulerAlgorithm::RoundRobin(2))
            .with_level(SchedulerAlgorithm::RoundRobin(4));

        // Schedule a single process.
        queue.schedule(Process::full(0, 6, OpCode::Inert));
        assert_eq!(queue.current_unchecked().id, 0);
        queue.current_unchecked().tick();
        queue.current_unchecked().tick();
        assert_eq!(queue.current_unchecked().id, 0);
        queue.current_unchecked().tick();
        queue.current_unchecked().tick();
        queue.current_unchecked().tick();
        queue.current_unchecked().tick();

        // We should be done.
        assert!(queue.current().is_none());
    }

    #[test]
    pub fn test_multilevel_preemptive() {
        // Form the multi-level feedback quuee.
        let mut queue = MultilevelQueue::new()
            .with_level(SchedulerAlgorithm::RoundRobin(2))
            .with_level(SchedulerAlgorithm::RoundRobin(4));

        // Schedule a single process.
        queue.schedule(Process::full(0, 6, OpCode::Inert));
        assert_eq!(queue.current_unchecked().id, 0);
        queue.current_unchecked().tick();
        queue.current_unchecked().tick();

        // Schedule another process. This should preempt the first one.
        queue.schedule(Process::full(1, 6, OpCode::Inert));
        assert_eq!(queue.current_unchecked().id, 1);
        queue.current_unchecked().tick();
        queue.current_unchecked().tick();

        // Now control should be handed back to the first process.
        assert_eq!(queue.current_unchecked().id, 0);
        queue.current_unchecked().tick_n(4);

        // First is done, should be the second.
        assert_eq!(queue.current_unchecked().id, 1);
        queue.current_unchecked().tick_n(4);

        // We should be done.
        assert!(queue.current().is_none());
    }
}