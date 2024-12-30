use std::{
    collections::HashMap,
    ops::{Add, Mul},
};
use thiserror::Error;

#[derive(Clone)]
pub struct MachineState<T> {
    memory: Vec<T>,
    sparse_memory: HashMap<usize, T>,
    pc: usize,
    relative_base: i32,
}

pub mod num {

    pub struct NumImpl<T, F, G>
    where
        F: Fn(T) -> Option<usize>,
        G: Fn(T) -> Option<i32>,
    {
        pub to_usize: F,
        pub to_i32: G,
        pub zero: T,
        pub one: T,
    }
    pub const fn i32() -> NumImpl<i32, impl Fn(i32) -> Option<usize>, impl Fn(i32) -> Option<i32>> {
        NumImpl {
            to_usize: |x| if x < 0 { None } else { Some(x as usize) },
            to_i32: |x| Some(x),
            zero: 0,
            one: 1,
        }
    }

    pub const fn i64() -> NumImpl<i64, impl Fn(i64) -> Option<usize>, impl Fn(i64) -> Option<i32>> {
        NumImpl {
            to_usize: |x| if x < 0 { None } else { Some(x as usize) },
            to_i32: |x| {
                if x < i32::MIN as i64 || x > i32::MAX as i64 {
                    None
                } else {
                    Some(x as i32)
                }
            },
            zero: 0,
            one: 1,
        }
    }

    pub const fn u64() -> NumImpl<u64, impl Fn(u64) -> Option<usize>, impl Fn(u64) -> Option<i32>> {
        NumImpl {
            to_usize: |x| Some(x as usize),
            to_i32: |x| {
                if x > i32::MAX as u64 {
                    None
                } else {
                    Some(x as i32)
                }
            },
            zero: 0,
            one: 1,
        }
    }

    pub const fn usize(
    ) -> NumImpl<usize, impl Fn(usize) -> Option<usize>, impl Fn(usize) -> Option<i32>> {
        NumImpl {
            to_usize: |x| Some(x),
            to_i32: |x| {
                if x > i32::MAX as usize {
                    None
                } else {
                    Some(x as i32)
                }
            },
            zero: 0,
            one: 1,
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
            sparse_memory: HashMap::new(),
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
            sparse_memory: HashMap::new(),
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

    // Pass the opcode so that we can work out the parameter modes.
    // This is neater because we can throw the right error if you give us
    // immediate-mode.
    fn process_binary_op<G, H, H2>(
        &mut self,
        opcode: usize,
        process: G,
        num: &num::NumImpl<T, H, H2>,
    ) -> Result<StepResult<T>, MachineExecutionError>
    where
        T: Copy,
        G: Fn(T, T) -> T,
        H: Fn(T) -> Option<usize>,
        H2: Fn(T) -> Option<i32>,
    {
        if opcode >= 100000 {
            return Err(MachineExecutionError::BadParameterMode(opcode));
        }

        let mode_1 = ParameterMode::of_int((opcode / 100) % 10)
            .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
        let mode_2 = ParameterMode::of_int((opcode / 1000) % 10)
            .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
        let arg_1 = self.read_param(self.pc + 1, mode_1, num)?;
        let arg_2 = self.read_param(self.pc + 2, mode_2, num)?;

        let result_pos = match ParameterMode::of_int((opcode / 10000) % 10)
            .ok_or(MachineExecutionError::BadParameterMode(opcode))?
        {
            ParameterMode::Relative => {
                let offset = (num.to_i32)(self.read_mem_elt(self.pc + 3, num)).ok_or(
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
            ParameterMode::Position => (num.to_usize)(self.read_mem_elt(self.pc + 3, num))
                .ok_or(MemoryAccessError::Negative)?,
            ParameterMode::Immediate => {
                return Err(MachineExecutionError::BadParameterMode(opcode))
            }
        };

        let result = process(arg_1, arg_2);

        self.set_mem_elt(result_pos, result);
        self.pc += 4;
        Ok(StepResult::Stepped)
    }

    pub fn one_step<G, H>(
        &mut self,
        num: &num::NumImpl<T, G, H>,
    ) -> Result<StepResult<T>, MachineExecutionError>
    where
        T: Add<T, Output = T> + Mul<T, Output = T> + Copy + std::cmp::Ord,
        G: Fn(T) -> Option<usize>,
        H: Fn(T) -> Option<i32>,
    {
        let opcode = self.read_mem_elt(self.pc, num);
        let opcode: usize = (num.to_usize)(opcode).ok_or(MachineExecutionError::OutOfBounds(
            MemoryAccessError::Negative,
        ))?;
        match opcode % 100 {
            1_usize => self.process_binary_op(opcode, |a, b| a + b, num),
            2 => self.process_binary_op(opcode, |a, b| a * b, num),
            3 => {
                if opcode >= 1000 {
                    return Err(MachineExecutionError::BadParameterMode(opcode));
                }
                let mode = ParameterMode::of_int((opcode / 100) % 10)
                    .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
                let location = match mode {
                    ParameterMode::Immediate => {
                        return Err(MachineExecutionError::BadParameterMode(opcode));
                    }
                    ParameterMode::Position => {
                        let location = self.read_mem_elt(self.pc + 1, num);
                        (num.to_usize)(location).ok_or(MemoryAccessError::Negative)?
                    }
                    ParameterMode::Relative => {
                        let offset = self.read_mem_elt(self.pc + 1, num);
                        let target = self.relative_base
                            + (num.to_i32)(offset).ok_or(MemoryAccessError::Overflow)?;
                        if target < 0 {
                            return Err(MachineExecutionError::OutOfBounds(
                                MemoryAccessError::Negative,
                            ));
                        }
                        target as usize
                    }
                };
                self.pc += 2;
                Ok(StepResult::Io(StepIoResult::AwaitingInput(location)))
            }
            4 => {
                if opcode >= 1000 {
                    return Err(MachineExecutionError::BadParameterMode(opcode));
                }
                let mode = ParameterMode::of_int((opcode / 100) % 10)
                    .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
                let to_output = self.read_param(self.pc + 1, mode, num)?;
                self.pc += 2;
                Ok(StepResult::Io(StepIoResult::Output(to_output)))
            }
            5 => {
                if opcode >= 10000 {
                    return Err(MachineExecutionError::BadParameterMode(opcode));
                }
                let mode_comparand = ParameterMode::of_int((opcode / 100) % 10)
                    .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
                let comparand = self.read_param(self.pc + 1, mode_comparand, num)?;
                if comparand != num.zero {
                    let mode_target = ParameterMode::of_int((opcode / 1000) % 10)
                        .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
                    let target = self.read_param(self.pc + 2, mode_target, num)?;
                    self.pc = (num.to_usize)(target).ok_or(MemoryAccessError::Negative)?;
                } else {
                    self.pc += 3;
                }
                Ok(StepResult::Stepped)
            }
            6 => {
                if opcode >= 10000 {
                    return Err(MachineExecutionError::BadParameterMode(opcode));
                }
                let mode_comparand = ParameterMode::of_int((opcode / 100) % 10)
                    .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
                let comparand = self.read_param(self.pc + 1, mode_comparand, num)?;
                if comparand == num.zero {
                    let mode_target = ParameterMode::of_int((opcode / 1000) % 10)
                        .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
                    let target = self.read_param(self.pc + 2, mode_target, num)?;
                    self.pc = (num.to_usize)(target).ok_or(MemoryAccessError::Negative)?;
                } else {
                    self.pc += 3;
                }
                Ok(StepResult::Stepped)
            }
            7 => {
                self.process_binary_op(opcode, |a, b| if a < b { num.one } else { num.zero }, num)?;
                Ok(StepResult::Stepped)
            }
            8 => {
                self.process_binary_op(
                    opcode,
                    |a, b| if a == b { num.one } else { num.zero },
                    num,
                )?;
                Ok(StepResult::Stepped)
            }
            9 => {
                if opcode >= 1000 {
                    return Err(MachineExecutionError::BadParameterMode(opcode));
                }
                let mode = ParameterMode::of_int((opcode / 100) % 10)
                    .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
                let value = self.read_param(self.pc + 1, mode, num)?;
                let increment = (num.to_i32)(value).ok_or(MemoryAccessError::Overflow)?;
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

    pub fn execute_until_input<G, H>(
        &mut self,
        num: &num::NumImpl<T, G, H>,
    ) -> Result<StepIoResult<T>, MachineExecutionError>
    where
        T: Add<T, Output = T> + Mul<T, Output = T> + Copy + Ord,
        G: Fn(T) -> Option<usize>,
        H: Fn(T) -> Option<i32>,
    {
        loop {
            match self.one_step(num)? {
                StepResult::Io(res) => {
                    return Ok(res);
                }
                StepResult::Stepped => {}
            }
        }
    }

    pub fn execute_to_end<G, H, I>(
        &mut self,
        get_input: &mut I,
        num: &num::NumImpl<T, G, H>,
    ) -> Result<Vec<T>, MachineExecutionError>
    where
        T: Add<T, Output = T> + Mul<T, Output = T> + Copy + Ord,
        I: Iterator<Item = T>,
        G: Fn(T) -> Option<usize>,
        H: Fn(T) -> Option<i32>,
    {
        let mut outputs = vec![];
        loop {
            match self.execute_until_input(num)? {
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
                        self.set_mem_elt(target_location, input);
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

    pub fn set_mem_elt(&mut self, i: usize, new_val: T) {
        if i < self.memory.len() {
            self.memory[i] = new_val;
        } else {
            self.sparse_memory.insert(i, new_val);
        }
    }

    pub fn read_mem_elt<G, H>(&self, i: usize, num: &num::NumImpl<T, G, H>) -> T
    where
        G: Fn(T) -> Option<usize>,
        H: Fn(T) -> Option<i32>,
        T: Clone,
    {
        if i < self.memory.len() {
            self.memory[i].clone()
        } else {
            match self.sparse_memory.get(&i) {
                None => num.zero.clone(),
                Some(entry) => entry.clone(),
            }
        }
    }

    fn read_param<G, H>(
        &self,
        i: usize,
        mode: ParameterMode,
        num: &num::NumImpl<T, G, H>,
    ) -> Result<T, MemoryAccessError>
    where
        T: Clone,
        G: Fn(T) -> Option<usize>,
        H: Fn(T) -> Option<i32>,
    {
        match mode {
            ParameterMode::Immediate => Ok(self.read_mem_elt(i, num)),
            ParameterMode::Position => {
                let pos = self.read_mem_elt(i, num);
                let pos = (num.to_usize)(pos);
                match pos {
                    None => Err(MemoryAccessError::Negative),
                    Some(pos) => Ok(self.read_mem_elt(pos, num)),
                }
            }
            ParameterMode::Relative => {
                let offset = self.read_mem_elt(i, num);
                let target =
                    self.relative_base + (num.to_i32)(offset).ok_or(MemoryAccessError::Overflow)?;
                if target >= 0 {
                    Ok(self.read_mem_elt(target as usize, num))
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

    fn assert_machines_eq<T, I, G, H, const N: usize>(
        initial: &[T; N],
        expected_memory: Option<&[T]>,
        input: &mut I,
        expected_output: &[T],
        num: &num::NumImpl<T, G, H>,
    ) where
        T: Add<T, Output = T> + Mul<T, Output = T> + Copy + Ord + std::fmt::Debug,
        I: Iterator<Item = T>,
        G: Fn(T) -> Option<usize>,
        H: Fn(T) -> Option<i32>,
    {
        let mut machine: MachineState<T> = MachineState::<T>::new_with_memory(initial);
        let output = machine.execute_to_end(input, num).unwrap();
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
            &num::usize(),
        );
    }

    #[test]
    fn day_2_2() {
        assert_machines_eq(
            &[2_usize, 3, 0, 3, 99],
            Some(&[2, 3, 0, 6, 99]),
            &mut std::iter::empty(),
            &[],
            &num::usize(),
        );
    }

    #[test]
    fn day_2_3() {
        assert_machines_eq(
            &[2_usize, 4, 4, 5, 99, 0],
            Some(&[2, 4, 4, 5, 99, 9801]),
            &mut std::iter::empty(),
            &[],
            &num::usize(),
        );
    }

    #[test]
    fn day_2_4() {
        assert_machines_eq(
            &[1_usize, 1, 1, 4, 99, 5, 6, 0, 99],
            Some(&[30, 1, 1, 4, 2, 5, 6, 0, 99]),
            &mut std::iter::empty(),
            &[],
            &num::usize(),
        );
    }

    #[test]
    fn day_5_1() {
        let program = [3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8];
        assert_machines_eq(&program, None, &mut std::iter::once(8), &[1], &num::i32());
        assert_machines_eq(&program, None, &mut std::iter::once(7), &[0], &num::i32());
    }

    #[test]
    fn day_5_2() {
        let program = [3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8];
        assert_machines_eq(&program, None, &mut std::iter::once(8), &[0], &num::i32());
        assert_machines_eq(&program, None, &mut std::iter::once(7), &[1], &num::i32());
        assert_machines_eq(&program, None, &mut std::iter::once(9), &[0], &num::i32());
    }

    #[test]
    fn day_5_3() {
        let program = [3, 3, 1108, -1, 8, 3, 4, 3, 99];
        assert_machines_eq(&program, None, &mut std::iter::once(8), &[1], &num::i32());
        assert_machines_eq(&program, None, &mut std::iter::once(7), &[0], &num::i32());
    }

    #[test]
    fn day_5_4() {
        let program = [3, 3, 1107, -1, 8, 3, 4, 3, 99];
        assert_machines_eq(&program, None, &mut std::iter::once(8), &[0], &num::i32());
        assert_machines_eq(&program, None, &mut std::iter::once(7), &[1], &num::i32());
        assert_machines_eq(&program, None, &mut std::iter::once(9), &[0], &num::i32());
    }

    #[test]
    fn day_5_6() {
        let program = [3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1];
        assert_machines_eq(&program, None, &mut std::iter::once(0), &[0], &num::i32());
        assert_machines_eq(&program, None, &mut std::iter::once(3), &[1], &num::i32());
    }

    #[test]
    fn day_5_7() {
        let program = [
            3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36, 98, 0,
            0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101, 1000, 1, 20, 4,
            20, 1105, 1, 46, 98, 99,
        ];
        assert_machines_eq(&program, None, &mut std::iter::once(7), &[999], &num::i32());
        assert_machines_eq(
            &program,
            None,
            &mut std::iter::once(8),
            &[1000],
            &num::i32(),
        );
        assert_machines_eq(
            &program,
            None,
            &mut std::iter::once(9),
            &[1001],
            &num::i32(),
        );
    }

    #[test]
    fn day_9_1() {
        let program = [
            109, 1, 204, -1, 1001, 100, 1, 100, 1008, 100, 16, 101, 1006, 101, 0, 99,
        ];
        assert_machines_eq(
            &program,
            None,
            &mut std::iter::empty(),
            &program,
            &num::i32(),
        );
    }

    #[test]
    fn day_9_2() {
        let program: [u64; 8] = [1102, 34915192, 34915192, 7, 4, 7, 99, 0];
        assert_machines_eq(
            &program,
            None,
            &mut std::iter::empty(),
            &[program[1] * program[2]],
            &num::u64(),
        );
    }

    #[test]
    fn day_9_3() {
        let program: [u64; 3] = [104, 1125899906842624, 99];
        assert_machines_eq(
            &program,
            None,
            &mut std::iter::empty(),
            &[program[1]],
            &num::u64(),
        );
    }
}
