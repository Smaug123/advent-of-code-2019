use std::ops::{Add, Mul};
use thiserror::Error;

#[derive(Clone)]
pub struct MachineState<T> {
    memory: Vec<T>,
    pc: usize,
    relative_base: i32,
}

pub trait Num {
    fn zero() -> Self;
    fn one() -> Self;
    fn to_usize(self) -> Option<usize>;
    fn to_i32(self) -> Option<i32>;
}

impl Num for i32 {
    fn zero() -> Self {
        0
    }

    fn one() -> Self {
        1
    }

    fn to_usize(self) -> Option<usize> {
        if self < 0 {
            None
        } else {
            Some(self as usize)
        }
    }

    fn to_i32(self) -> Option<i32> {
        Some(self)
    }
}

impl Num for u64 {
    fn zero() -> Self {
        0
    }

    fn one() -> Self {
        1
    }

    fn to_usize(self) -> Option<usize> {
        Some(self as usize)
    }

    fn to_i32(self) -> Option<i32> {
        if self <= (i32::MAX as u64) {
            Some(self as i32)
        } else {
            None
        }
    }
}

impl Num for usize {
    fn zero() -> Self {
        0
    }

    fn one() -> Self {
        1
    }

    fn to_usize(self) -> Option<usize> {
        Some(self)
    }

    fn to_i32(self) -> Option<i32> {
        if self <= (i32::MAX as usize) {
            Some(self as i32)
        } else {
            None
        }
    }
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
    #[error("attempted to apply memory index offset too big to store")]
    Overflow,
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

pub enum StepIoResult<T> {
    // Machine has terminated.
    Terminated,
    // Machine has emitted a single output.
    Output(T),
    // Get an input and store it in this location.
    AwaitingInput(usize),
}

pub enum StepResult<T> {
    // Machine has executed an instruction, with no I/O.
    Stepped,
    // Machine has executed an instruction and performed output, or is blocked on input, or has finished.
    Io(StepIoResult<T>),
}

impl<T> From<StepIoResult<T>> for StepResult<T> {
    fn from(value: StepIoResult<T>) -> Self {
        StepResult::Io(value)
    }
}

enum ParameterMode {
    Immediate,
    Position,
    Relative,
}

impl ParameterMode {
    const fn of_int(u: usize) -> Option<ParameterMode> {
        match u {
            0 => Some(ParameterMode::Position),
            1 => Some(ParameterMode::Immediate),
            2 => Some(ParameterMode::Relative),
            _ => None,
        }
    }
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
            relative_base: 0,
        }
    }

    pub fn new_with_memory<J>(mem: &J) -> MachineState<T>
    where
        J: IntoIterator<Item = T>,
        J: Clone,
    {
        MachineState {
            memory: mem.clone().into_iter().collect(),
            pc: 0,
            relative_base: 0,
        }
    }

    pub fn reset<J>(&mut self, mem: J)
    where
        J: IntoIterator<Item = T> + Clone,
    {
        self.pc = 0;
        self.memory.clear();
        self.memory.extend(mem);
    }

    fn consume_args_2(&self, opcode: usize) -> Result<(T, T), MachineExecutionError>
    where
        T: Copy + Num,
    {
        if opcode >= 10000 {
            return Err(MachineExecutionError::BadParameterMode(opcode));
        }
        let mode_1 = ParameterMode::of_int((opcode / 100) % 10)
            .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
        let mode_2 = ParameterMode::of_int((opcode / 1000) % 10)
            .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
        let arg1 = *self.read_param(self.pc + 1, mode_1)?;
        let arg2 = *self.read_param(self.pc + 2, mode_2)?;
        Ok((arg1, arg2))
    }

    fn consume_args_1(&self, opcode: usize) -> Result<T, MachineExecutionError>
    where
        T: Copy + Num,
    {
        if opcode >= 1000 {
            return Err(MachineExecutionError::BadParameterMode(opcode));
        }
        let mode = ParameterMode::of_int((opcode / 100) % 10)
            .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
        let to_output = *self.read_param(self.pc + 1, mode)?;
        Ok(to_output)
    }

    fn transform_to_dest<F>(
        &mut self,
        opcode: usize,
        f: F,
    ) -> Result<StepResult<T>, MachineExecutionError>
    where
        T: Copy + Num,
        F: Fn(T, T) -> T,
    {
        if opcode >= 100000 {
            return Err(MachineExecutionError::BadParameterMode(opcode));
        }
        let mode_1 = ParameterMode::of_int((opcode / 100) % 10)
            .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
        let mode_2 = ParameterMode::of_int((opcode / 1000) % 10)
            .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
        let result_pos = match ParameterMode::of_int((opcode / 10000) % 10)
            .ok_or(MachineExecutionError::BadParameterMode(opcode))?
        {
            ParameterMode::Position => {
                T::to_usize(*self.read_mem_elt(self.pc + 3)?).ok_or(MemoryAccessError::Negative)?
            }
            ParameterMode::Relative => {
                let offset = T::to_i32(*self.read_mem_elt(self.pc + 3)?).ok_or(
                    MachineExecutionError::OutOfBounds(MemoryAccessError::Negative),
                )?;
                let target = self.relative_base + offset;
                if target < 0 {
                    return Err(MachineExecutionError::OutOfBounds(
                        MemoryAccessError::Negative,
                    ));
                }
                target as usize
            }
            ParameterMode::Immediate => {
                return Err(MachineExecutionError::BadParameterMode(opcode))
            }
        };
        let arg1 = *self.read_param(self.pc + 1, mode_1)?;
        let arg2 = *self.read_param(self.pc + 2, mode_2)?;
        let result = f(arg1, arg2);
        self.set_mem_elt(result_pos, result)?;
        self.pc += 4;
        Ok(StepResult::Stepped)
    }

    pub fn one_step(&mut self) -> Result<StepResult<T>, MachineExecutionError>
    where
        T: Add<T, Output = T> + Mul<T, Output = T> + Copy + std::cmp::Ord + Num,
    {
        let opcode = *self.read_mem_elt(self.pc)?;
        let opcode: usize = T::to_usize(opcode).ok_or(MachineExecutionError::OutOfBounds(
            MemoryAccessError::Negative,
        ))?;
        match opcode % 100 {
            1_usize => self.transform_to_dest(opcode, |a, b| a + b),
            2 => self.transform_to_dest(opcode, |a, b| a * b),
            3 => {
                if opcode != 3 {
                    return Err(MachineExecutionError::BadParameterMode(opcode));
                }
                let location = self.read_mem_elt(self.pc + 1)?;
                let location = T::to_usize(*location).ok_or(MemoryAccessError::Negative)?;
                self.pc += 2;
                Ok(StepResult::Io(StepIoResult::AwaitingInput(location)))
            }
            4 => {
                let to_output = self.consume_args_1(opcode)?;
                self.pc += 2;
                Ok(StepResult::Io(StepIoResult::Output(to_output)))
            }
            5 => {
                let (comparand, target) = self.consume_args_2(opcode)?;
                if comparand != T::zero() {
                    self.pc = T::to_usize(target).ok_or(MemoryAccessError::Negative)?;
                } else {
                    self.pc += 3;
                }
                Ok(StepResult::Stepped)
            }
            6 => {
                let (comparand, target) = self.consume_args_2(opcode)?;
                if comparand == T::zero() {
                    self.pc = T::to_usize(target).ok_or(MemoryAccessError::Negative)?;
                } else {
                    self.pc += 3;
                }
                Ok(StepResult::Stepped)
            }
            7 => self.transform_to_dest(opcode, |a, b| if a < b { T::one() } else { T::zero() }),
            8 => self.transform_to_dest(opcode, |a, b| if a == b { T::one() } else { T::zero() }),
            9 => {
                let arg = self.consume_args_1(opcode)?;
                let increment = T::to_i32(arg).ok_or(MemoryAccessError::Overflow)?;
                self.relative_base += increment;
                self.pc += 2;
                Ok(StepResult::Stepped)
            }
            99 => {
                if opcode != 99 {
                    return Err(MachineExecutionError::BadParameterMode(opcode));
                }
                Ok(StepResult::Io(StepIoResult::Terminated))
            }
            bad => Err(MachineExecutionError::BadOpcode(bad, self.pc)),
        }
    }

    pub fn execute_until_input(&mut self) -> Result<StepIoResult<T>, MachineExecutionError>
    where
        T: Add<T, Output = T> + Mul<T, Output = T> + Copy + Ord + Num,
    {
        loop {
            match self.one_step()? {
                StepResult::Io(res) => {
                    return Ok(res);
                }
                StepResult::Stepped => {}
            }
        }
    }

    pub fn execute_to_end<I>(&mut self, get_input: &mut I) -> Result<Vec<T>, MachineExecutionError>
    where
        T: Add<T, Output = T> + Mul<T, Output = T> + Copy + Ord + Num,
        I: Iterator<Item = T>,
    {
        let mut outputs = vec![];
        loop {
            match self.execute_until_input()? {
                StepIoResult::Terminated => {
                    return Ok(outputs);
                }
                StepIoResult::Output(output) => {
                    outputs.push(output);
                }
                StepIoResult::AwaitingInput(target_location) => match get_input.next() {
                    None => {
                        return Err(MachineExecutionError::NoInput);
                    }
                    Some(input) => {
                        self.set_mem_elt(target_location, input)?;
                    }
                },
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

    fn read_param(&self, i: usize, mode: ParameterMode) -> Result<&T, MemoryAccessError>
    where
        T: Copy + Num,
    {
        match mode {
            ParameterMode::Immediate => self.read_mem_elt(i),
            ParameterMode::Position => {
                let pos = self.read_mem_elt(i)?;
                let pos = T::to_usize(*pos);
                match pos {
                    None => Err(MemoryAccessError::Negative),
                    Some(pos) => self.read_mem_elt(pos),
                }
            }
            ParameterMode::Relative => {
                let offset = *self.read_mem_elt(i)?;
                let target =
                    self.relative_base + T::to_i32(offset).ok_or(MemoryAccessError::Overflow)?;
                if target >= 0 {
                    self.read_mem_elt(target as usize)
                } else {
                    Err(MemoryAccessError::Negative)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_machines_eq<T, I, const N: usize>(
        initial: &[T; N],
        expected_memory: Option<&[T]>,
        input: &mut I,
        expected_output: &[T],
    ) where
        T: Add<T, Output = T> + Mul<T, Output = T> + Copy + Ord + Num,
        I: Iterator<Item = T>,
    {
        let mut machine: MachineState<T> = MachineState::<T>::new_with_memory(initial);
        let output = machine.execute_to_end(input).unwrap();
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
            &mut std::iter::empty(),
            &[],
        );
    }

    #[test]
    fn day_2_2() {
        assert_machines_eq(
            &[2_usize, 3, 0, 3, 99],
            Some(&[2, 3, 0, 6, 99]),
            &mut std::iter::empty(),
            &[],
        );
    }

    #[test]
    fn day_2_3() {
        assert_machines_eq(
            &[2_usize, 4, 4, 5, 99, 0],
            Some(&[2, 4, 4, 5, 99, 9801]),
            &mut std::iter::empty(),
            &[],
        );
    }

    #[test]
    fn day_2_4() {
        assert_machines_eq(
            &[1_usize, 1, 1, 4, 99, 5, 6, 0, 99],
            Some(&[30, 1, 1, 4, 2, 5, 6, 0, 99]),
            &mut std::iter::empty(),
            &[],
        );
    }

    #[test]
    fn day_5_1() {
        let program = [3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8];
        assert_machines_eq(&program, None, &mut std::iter::once(8), &[1]);
        assert_machines_eq(&program, None, &mut std::iter::once(7), &[0]);
    }

    #[test]
    fn day_5_2() {
        let program = [3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8];
        assert_machines_eq(&program, None, &mut std::iter::once(8), &[0]);
        assert_machines_eq(&program, None, &mut std::iter::once(7), &[1]);
        assert_machines_eq(&program, None, &mut std::iter::once(9), &[0]);
    }

    #[test]
    fn day_5_3() {
        let program = [3, 3, 1108, -1, 8, 3, 4, 3, 99];
        assert_machines_eq(&program, None, &mut std::iter::once(8), &[1]);
        assert_machines_eq(&program, None, &mut std::iter::once(7), &[0]);
    }

    #[test]
    fn day_5_4() {
        let program = [3, 3, 1107, -1, 8, 3, 4, 3, 99];
        assert_machines_eq(&program, None, &mut std::iter::once(8), &[0]);
        assert_machines_eq(&program, None, &mut std::iter::once(7), &[1]);
        assert_machines_eq(&program, None, &mut std::iter::once(9), &[0]);
    }

    #[test]
    fn day_5_6() {
        let program = [3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1];
        assert_machines_eq(&program, None, &mut std::iter::once(0), &[0]);
        assert_machines_eq(&program, None, &mut std::iter::once(3), &[1]);
    }

    #[test]
    fn day_5_7() {
        let program = [
            3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36, 98, 0,
            0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101, 1000, 1, 20, 4,
            20, 1105, 1, 46, 98, 99,
        ];
        assert_machines_eq(&program, None, &mut std::iter::once(7), &[999]);
        assert_machines_eq(&program, None, &mut std::iter::once(8), &[1000]);
        assert_machines_eq(&program, None, &mut std::iter::once(9), &[1001]);
    }

    //#[test]
    //fn day_9_1() {
    //    let program = [
    //        109, 1, 204, -1, 1001, 100, 1, 100, 1008, 100, 16, 101, 1006, 101, 0, 99,
    //    ];
    //    assert_machines_eq(
    //        &program,
    //        None,
    //        &mut std::iter::empty(),
    //        &program,
    //    );
    //}

    #[test]
    fn day_9_2() {
        let program: [u64; 8] = [1102, 34915192, 34915192, 7, 4, 7, 99, 0];
        assert_machines_eq(
            &program,
            None,
            &mut std::iter::empty(),
            &[program[1] * program[2]],
        );
    }

    #[test]
    fn day_9_3() {
        let program: [u64; 3] = [104, 1125899906842624, 99];
        assert_machines_eq(&program, None, &mut std::iter::empty(), &[program[1]]);
    }
}
