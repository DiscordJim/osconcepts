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
    /// Adds a new level to the multi-level queue. This is to be used
    /// sort of like a builder pattern.
    pub fn with_level(mut self, level: SchedulerAlgorithm) -> Self {
        // Create a new scheduler with the algorithm and turn on feedback mode.
        self.levels.push_back(Scheduler::new(level).with_feedback());
        self
    }
    /// Schedules a new task into the topmost queue.
    /// 
    /// # Panics
    /// This will panic if the queue has no levels.
    pub fn schedule(&mut self, mut process: Process) {
        let mut point = 0;

        // Keep shifting things down the queue until the queue is good.
        while let Some(inner) = self.levels[point].schedule(process) {
            process = inner.proc;
            if point != self.levels.len() - 1 {
                point += 1;
            }
          
        }

        // let resul = self.levels[0].schedule(process);

        // println!("Result: {:?}", resul);
    }
    /// Gets the current scheduled task.
    /// 
    /// # Panics
    /// If there is no current task.
    pub fn current_unchecked(&mut self) -> &mut ProcessRecord {
        self.current().unwrap()
    }
    
    /// Gets the currently scheduled task from
    /// the highest possible queue.
    pub fn current(&mut self) -> Option<&mut ProcessRecord> {
        self.current_with_key().map(|f|f.1)
    }
    pub fn current_with_key(&mut self) -> Option<(usize, &mut ProcessRecord)> {
        // let mut current = None;
        // let mut point = 0;
        // while let Some(inner) = self.levels[point].fetch_current().1 {
        //     current = Some(inner.proc);
        //     point += 1;
        // }

        for level in 0..self.levels.len() {
            if level < self.levels.len() - 1 {
                if let Some(bumped) = self.levels[level].fetch_current().1 {
                    self.levels[level + 1].schedule(bumped.proc);
                }
            } else {
                // Last level, just put it back in.
                if let Some(bumped) = self.levels[level].fetch_current().1 {
                    self.levels[level].schedule(bumped.proc);
                }
            }
        }


        for i in 0..self.levels.len() {


            if self.levels[i].current().is_some() {
                return Some((i, self.levels[i].current()?));
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
            .with_level(SchedulerAlgorithm::RoundRobin(4))
            .with_level(SchedulerAlgorithm::FirstComeFirstServe);

        // Schedule a single process. This should land on the first level.
        queue.schedule(Process::full(0, 8, OpCode::Inert));
        assert_eq!(queue.current_unchecked().id, 0);
        assert_eq!(queue.current_with_key().unwrap().0, 0);
        queue.current_unchecked().tick_n(2);

        // The above should have elapsed its time on the first queue and thus
        // should be in the queue with the ID of one.
        let (level, proc) = queue.current_with_key().unwrap();
        assert_eq!(proc.id, 0);
        assert_eq!(level, 1);

        // Now we tick it.
        queue.current_unchecked().tick_n(4);
        let (level, proc) = queue.current_with_key().unwrap();
        assert_eq!(proc.id, 0);
        assert_eq!(level, 2);
        queue.current_unchecked().tick_n(2);

        // We should be done.
        assert!(queue.current().is_none());
    }

    #[test]
    pub fn test_multilevel_preemptive() {
        // Form the multi-level feedback quuee.
        let mut queue = MultilevelQueue::new()
            .with_level(SchedulerAlgorithm::RoundRobin(2))
            .with_level(SchedulerAlgorithm::RoundRobin(4))
            .with_level(SchedulerAlgorithm::FirstComeFirstServe);

        // Schedule a single process.
        queue.schedule(Process::full(0, 6, OpCode::Inert));
        let (level, proc) = queue.current_with_key().unwrap();
        assert_eq!(proc.id, 0);
        assert_eq!(level, 0);
        queue.current_unchecked().tick_n(2);

        // Schedule another process. This should preempt the first one.
        queue.schedule(Process::full(1, 6, OpCode::Inert));
        let (level, proc) = queue.current_with_key().unwrap();
        assert_eq!(proc.id, 1);
        assert_eq!(level, 0);
        queue.current_unchecked().tick_n(2);

        // Now control should be handed back to the first process.
        let (level, proc) = queue.current_with_key().unwrap();
        assert_eq!(proc.id, 0);
        assert_eq!(level, 1);
        
        queue.current_unchecked().tick_n(4);

        // First is done, should be the second.
        let (level, proc) = queue.current_with_key().unwrap();
        assert_eq!(proc.id, 1);
        assert_eq!(level, 1);
        queue.current_unchecked().tick_n(4);

        // We should be done.
        assert!(queue.current().is_none());
    }
}