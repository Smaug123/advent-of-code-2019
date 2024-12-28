use std::ops::{Add, Mul};
use thiserror::Error;

pub struct MachineState<T, I> {
    memory: Vec<T>,
    pc: usize,
    get_input: I,
}

#[derive(Error, Debug)]
#[error(
    "attempted to access position {pos} but memory only has length {len} (is_write: {is_write})"
)]
pub struct MemoryAccessTooFarError {
    pos: usize,
    len: usize,
    is_write: bool,
}

#[derive(Error, Debug)]
pub enum MemoryAccessError {
    #[error(transparent)]
    TooFar(#[from] MemoryAccessTooFarError),
    #[error("attempted to access negative memory index")]
    Negative,
}

#[derive(Error, Debug)]
pub enum MachineExecutionError {
    #[error("instruction {0} at position {1} unrecognised")]
    BadOpcode(usize, usize),
    #[error(transparent)]
    OutOfBounds(#[from] MemoryAccessError),
    #[error("input requested but no input provided")]
    NoInput,
    #[error("invalid parameter mode {0}")]
    BadParameterMode(usize),
}

pub enum StepResult<T> {
    Terminated,
    Processed(Option<T>),
}

enum ParameterMode {
    Immediate,
    Position,
}

impl ParameterMode {
    const fn of_int(u: usize) -> Option<ParameterMode> {
        match u {
            0 => Some(ParameterMode::Position),
            1 => Some(ParameterMode::Immediate),
            _ => None,
        }
    }
}

impl<T, I> MachineState<T, I> {
    pub fn new(get_input: I) -> MachineState<T, I>
    where
        I: Iterator<Item = T>,
    {
        MachineState {
            memory: vec![],
            pc: 0,
            get_input,
        }
    }

    pub fn new_with_memory<J>(mem: &J, get_input: I) -> MachineState<T, I>
    where
        J: IntoIterator<Item = T>,
        J: Clone,
    {
        MachineState {
            memory: mem.clone().into_iter().collect(),
            pc: 0,
            get_input,
        }
    }

    pub fn reset_memory<J>(&mut self, mem: J)
    where
        J: IntoIterator<Item = T> + Clone,
    {
        self.pc = 0;
        self.memory.clear();
        self.memory.extend(mem);
    }

    fn process_binary_op<G, H>(
        &mut self,
        mode_1: ParameterMode,
        mode_2: ParameterMode,
        process: G,
        to_usize: &H,
    ) -> Result<StepResult<T>, MachineExecutionError>
    where
        T: Copy,
        G: Fn(T, T) -> T,
        H: Fn(T) -> Option<usize>,
    {
        let arg_1 = *self.read_param(self.pc + 1, mode_1, to_usize)?;
        let arg_2 = *self.read_param(self.pc + 2, mode_2, to_usize)?;

        let result_pos = *self.read_mem_elt(self.pc + 3)?;

        self.set_mem_elt(
            to_usize(result_pos).ok_or(MemoryAccessError::Negative)?,
            process(arg_1, arg_2),
        )?;
        self.pc += 4;
        Ok(StepResult::Processed(None))
    }

    pub fn one_step<G>(&mut self, to_usize: &G) -> Result<StepResult<T>, MachineExecutionError>
    where
        T: Add<T, Output = T> + Mul<T, Output = T> + Copy,
        I: Iterator<Item = T>,
        G: Fn(T) -> Option<usize>,
    {
        let opcode = *self.read_mem_elt(self.pc)?;
        let opcode: usize = to_usize(opcode).ok_or(MachineExecutionError::OutOfBounds(
            MemoryAccessError::Negative,
        ))?;
        match opcode % 100 {
            1_usize => {
                if opcode >= 10000 {
                    return Err(MachineExecutionError::BadParameterMode(opcode));
                }
                let mode_1 = ParameterMode::of_int((opcode / 100) % 10)
                    .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
                let mode_2 = ParameterMode::of_int((opcode / 1000) % 10)
                    .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
                self.process_binary_op(mode_1, mode_2, |a, b| a + b, to_usize)
            }
            2_usize => {
                if opcode >= 10000 {
                    return Err(MachineExecutionError::BadParameterMode(opcode));
                }
                let mode_1 = ParameterMode::of_int((opcode / 100) % 10)
                    .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
                let mode_2 = ParameterMode::of_int((opcode / 1000) % 10)
                    .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
                self.process_binary_op(mode_1, mode_2, |a, b| a * b, to_usize)
            }
            3_usize => {
                if opcode != 3 {
                    return Err(MachineExecutionError::BadParameterMode(opcode));
                }
                let location = self.read_mem_elt(self.pc + 1)?;
                let location = to_usize(*location).ok_or(MemoryAccessError::Negative)?;
                let input = match self.get_input.next() {
                    None => {
                        return Err(MachineExecutionError::NoInput);
                    }
                    Some(v) => v,
                };
                self.set_mem_elt(location, input)?;
                self.pc += 2;
                Ok(StepResult::Processed(None))
            }
            4_usize => {
                if opcode >= 1000 {
                    return Err(MachineExecutionError::BadParameterMode(opcode));
                }
                let mode = ParameterMode::of_int((opcode / 100) % 10)
                    .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
                let to_output = match mode {
                    ParameterMode::Position => {
                        let addr = to_usize(*self.read_mem_elt(self.pc + 1)?)
                            .ok_or(MemoryAccessError::Negative)?;
                        *self.read_mem_elt(addr)?
                    }
                    ParameterMode::Immediate => *self.read_mem_elt(self.pc + 1)?,
                };
                self.pc += 2;
                Ok(StepResult::Processed(Some(to_output)))
            }
            99_usize => {
                if opcode != 99 {
                    return Err(MachineExecutionError::BadParameterMode(opcode));
                }
                Ok(StepResult::Terminated)
            }
            bad => Err(MachineExecutionError::BadOpcode(bad, self.pc)),
        }
    }

    pub fn execute_to_end<G>(&mut self, to_usize: &G) -> Result<Vec<T>, MachineExecutionError>
    where
        T: Add<T, Output = T> + Mul<T, Output = T> + Copy,
        I: Iterator<Item = T>,
        G: Fn(T) -> Option<usize>,
    {
        let mut outputs = vec![];
        loop {
            match self.one_step(to_usize)? {
                StepResult::Terminated => {
                    return Ok(outputs);
                }
                StepResult::Processed(Some(output)) => {
                    outputs.push(output);
                }
                StepResult::Processed(None) => {}
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
            Err(MemoryAccessError::TooFar(MemoryAccessTooFarError {
                pos: i,
                len: self.memory.len(),
                is_write: true,
            }))
        }
    }

    pub fn read_mem_elt(&self, i: usize) -> Result<&T, MemoryAccessError> {
        if i < self.memory.len() {
            Ok(&self.memory[i])
        } else {
            Err(MemoryAccessError::TooFar(MemoryAccessTooFarError {
                pos: i,
                len: self.memory.len(),
                is_write: false,
            }))
        }
    }

    fn read_param<G>(
        &self,
        i: usize,
        mode: ParameterMode,
        to_usize: &G,
    ) -> Result<&T, MemoryAccessError>
    where
        T: Copy,
        G: Fn(T) -> Option<usize>,
    {
        match mode {
            ParameterMode::Immediate => self.read_mem_elt(i),
            ParameterMode::Position => {
                let pos = self.read_mem_elt(i)?;
                let pos = to_usize(*pos);
                match pos {
                    None => Err(MemoryAccessError::Negative),
                    Some(pos) => self.read_mem_elt(pos),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_machines_eq<T, I, G, const N: usize>(
        initial: &[T; N],
        expected_memory: Option<&[T]>,
        input: I,
        expected_output: &[T],
        to_usize: &G,
    ) where
        T: Add<T, Output = T> + Mul<T, Output = T> + Copy + PartialEq,
        I: Iterator<Item = T>,
        G: Fn(T) -> Option<usize>,
    {
        let mut machine: MachineState<T, _> = MachineState::<T, _>::new_with_memory(initial, input);
        let output = machine.execute_to_end(to_usize).unwrap();
        match expected_memory {
            None => {}
            Some(expected_memory) => {
                assert!(machine.dump_memory().eq(expected_memory.iter().copied()));
            }
        }
        assert!(output.iter().copied().eq(expected_output.iter().copied()));
    }

    #[test]
    fn day_2_1() {
        assert_machines_eq(
            &[1_usize, 9, 10, 3, 2, 3, 11, 0, 99, 30, 40, 50],
            Some(&[3500, 9, 10, 70, 2, 3, 11, 0, 99, 30, 40, 50]),
            std::iter::empty(),
            &[],
            &|x| Some(x),
        );
    }

    #[test]
    fn day_2_2() {
        assert_machines_eq(
            &[2_usize, 3, 0, 3, 99],
            Some(&[2, 3, 0, 6, 99]),
            std::iter::empty(),
            &[],
            &|x| Some(x),
        );
    }

    #[test]
    fn day_2_3() {
        assert_machines_eq(
            &[2_usize, 4, 4, 5, 99, 0],
            Some(&[2, 4, 4, 5, 99, 9801]),
            std::iter::empty(),
            &[],
            &|x| Some(x),
        );
    }

    #[test]
    fn day_2_4() {
        assert_machines_eq(
            &[1_usize, 1, 1, 4, 99, 5, 6, 0, 99],
            Some(&[30, 1, 1, 4, 2, 5, 6, 0, 99]),
            std::iter::empty(),
            &[],
            &|x| Some(x),
        );
    }

    #[test]
    fn day_5_1() {
        let program = [3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8];
        assert_machines_eq(&program, None, std::iter::once(8), &[1], &|x| {
            if x < 0 {
                None
            } else {
                Some(x as usize)
            }
        });
        assert_machines_eq(&program, None, std::iter::once(7), &[0], &|x| {
            if x < 0 {
                None
            } else {
                Some(x as usize)
            }
        });
    }

    #[test]
    fn day_5_2() {
        let program = [3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8];
        assert_machines_eq(&program, None, std::iter::once(8), &[0], &|x| {
            if x < 0 {
                None
            } else {
                Some(x as usize)
            }
        });
        assert_machines_eq(&program, None, std::iter::once(7), &[1], &|x| {
            if x < 0 {
                None
            } else {
                Some(x as usize)
            }
        });
        assert_machines_eq(&program, None, std::iter::once(9), &[0], &|x| {
            if x < 0 {
                None
            } else {
                Some(x as usize)
            }
        });
    }

    #[test]
    fn day_5_3() {
        let program = [3, 3, 1108, -1, 8, 3, 4, 3, 99];
        assert_machines_eq(&program, None, std::iter::once(8), &[1], &|x| {
            if x < 0 {
                None
            } else {
                Some(x as usize)
            }
        });
        assert_machines_eq(&program, None, std::iter::once(7), &[0], &|x| {
            if x < 0 {
                None
            } else {
                Some(x as usize)
            }
        });
    }

    #[test]
    fn day_5_4() {
        let program = [3, 3, 1107, -1, 8, 3, 4, 3, 99];
        assert_machines_eq(&program, None, std::iter::once(8), &[0], &|x| {
            if x < 0 {
                None
            } else {
                Some(x as usize)
            }
        });
        assert_machines_eq(&program, None, std::iter::once(7), &[1], &|x| {
            if x < 0 {
                None
            } else {
                Some(x as usize)
            }
        });
        assert_machines_eq(&program, None, std::iter::once(9), &[0], &|x| {
            if x < 0 {
                None
            } else {
                Some(x as usize)
            }
        });
    }

    #[test]
    fn day_5_6() {
        let program = [3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1];
        assert_machines_eq(&program, None, std::iter::once(0), &[0], &|x| {
            if x < 0 {
                None
            } else {
                Some(x as usize)
            }
        });
        assert_machines_eq(&program, None, std::iter::once(3), &[1], &|x| {
            if x < 0 {
                None
            } else {
                Some(x as usize)
            }
        });
    }

    #[test]
    fn day_5_7() {
        let program = [
            3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36, 98, 0,
            0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101, 1000, 1, 20, 4,
            20, 1105, 1, 46, 98, 99,
        ];
        assert_machines_eq(&program, None, std::iter::once(7), &[999], &|x| {
            if x < 0 {
                None
            } else {
                Some(x as usize)
            }
        });
        assert_machines_eq(&program, None, std::iter::once(8), &[1000], &|x| {
            if x < 0 {
                None
            } else {
                Some(x as usize)
            }
        });
        assert_machines_eq(&program, None, std::iter::once(9), &[1001], &|x| {
            if x < 0 {
                None
            } else {
                Some(x as usize)
            }
        });
    }
}
