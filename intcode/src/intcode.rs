use std::convert::Into;
use std::ops::{Add, Mul};
use thiserror::Error;

pub struct MachineState<T> {
    memory: Vec<T>,
    pc: usize,
}

#[derive(Error, Debug)]
#[error(
    "attempted to access position {pos} but memory only has length {len} (is_write: {is_write})"
)]
pub struct MemoryAccessError {
    pos: usize,
    len: usize,
    is_write: bool,
}

#[derive(Error, Debug)]
pub enum MachineExecutionError {
    #[error("instruction {0} at position {1} unrecognised")]
    BadOpcode(usize, usize),
    #[error(transparent)]
    OutOfBounds(#[from] MemoryAccessError),
}

pub enum StepResult {
    Terminated,
    Processed,
}

impl<T> Default for MachineState<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> MachineState<T> {
    pub fn new() -> MachineState<T> {
        MachineState {
            memory: vec![],
            pc: 0,
        }
    }

    pub fn new_with_memory<I>(mem: &I) -> MachineState<T>
    where
        I: IntoIterator<Item = T>,
        I: Clone,
    {
        MachineState {
            memory: mem.clone().into_iter().collect(),
            pc: 0,
        }
    }

    pub fn reset_memory<I>(&mut self, mem: I)
    where
        I: IntoIterator<Item = T> + Clone,
    {
        self.pc = 0;
        self.memory.clear();
        self.memory.extend(mem);
    }

    fn process_binary_op<F>(&mut self, process: F) -> Result<StepResult, MachineExecutionError>
    where
        T: Into<usize> + Copy,
        F: FnOnce(T, T) -> T,
    {
        let result_pos = *self.read_mem_elt(self.pc + 3)?;
        let arg1_pos = *self.read_mem_elt(self.pc + 1)?;
        let arg2_pos = *self.read_mem_elt(self.pc + 2)?;

        let arg1 = *self.read_mem_elt(arg1_pos.into())?;
        let arg2 = *self.read_mem_elt(arg2_pos.into())?;
        self.set_mem_elt(result_pos.into(), process(arg1, arg2))?;
        self.pc += 4;
        Ok(StepResult::Processed)
    }

    pub fn one_step(&mut self) -> Result<StepResult, MachineExecutionError>
    where
        T: Add<T, Output = T> + Mul<T, Output = T> + Into<usize> + Copy,
    {
        match (*self.read_mem_elt(self.pc)?).into() {
            1_usize => self.process_binary_op(|a, b| a + b),
            2_usize => self.process_binary_op(|a, b| a * b),
            99_usize => Ok(StepResult::Terminated),
            bad => Err(MachineExecutionError::BadOpcode(bad, self.pc)),
        }
    }

    pub fn execute_to_end(&mut self) -> Result<(), MachineExecutionError>
    where
        T: Add<T, Output = T> + Mul<T, Output = T> + Into<usize> + Copy,
    {
        loop {
            match self.one_step()? {
                StepResult::Terminated => {
                    return Ok(());
                }
                StepResult::Processed => {}
            }
        }
    }

    pub fn dump_memory(&self) -> impl Iterator<Item = T> + '_
    where
        T: Copy,
    {
        self.memory.iter().copied()
    }

    pub fn set_mem_elt(&mut self, i: usize, new_val: T) -> Result<(), MemoryAccessError> {
        if i < self.memory.len() {
            self.memory[i] = new_val;
            Ok(())
        } else {
            Err(MemoryAccessError {
                pos: i,
                len: self.memory.len(),
                is_write: true,
            })
        }
    }

    pub fn read_mem_elt(&mut self, i: usize) -> Result<&T, MemoryAccessError> {
        if i < self.memory.len() {
            Ok(&self.memory[i])
        } else {
            Err(MemoryAccessError {
                pos: i,
                len: self.memory.len(),
                is_write: false,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_machines_eq<T, const N: usize, const M: usize>(initial: &[T; N], expected: &[T; M])
    where
        T: Add<T, Output = T> + Mul<T, Output = T> + Into<usize> + Copy + PartialEq,
    {
        let mut machine: MachineState<T> = MachineState::<T>::new_with_memory(initial);
        machine.execute_to_end().unwrap();
        assert!(machine.dump_memory().eq(expected.iter().copied()));
    }

    #[test]
    fn day_2_1() {
        assert_machines_eq(
            &[1_usize, 9, 10, 3, 2, 3, 11, 0, 99, 30, 40, 50],
            &[3500, 9, 10, 70, 2, 3, 11, 0, 99, 30, 40, 50],
        );
    }

    #[test]
    fn day_2_2() {
        assert_machines_eq(&[2_usize, 3, 0, 3, 99], &[2, 3, 0, 6, 99]);
    }

    #[test]
    fn day_2_3() {
        assert_machines_eq(&[2_usize, 4, 4, 5, 99, 0], &[2, 4, 4, 5, 99, 9801]);
    }

    #[test]
    fn day_2_4() {
        assert_machines_eq(
            &[1_usize, 1, 1, 4, 99, 5, 6, 0, 99],
            &[30, 1, 1, 4, 2, 5, 6, 0, 99],
        );
    }
}
