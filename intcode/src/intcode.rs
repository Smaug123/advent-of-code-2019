use std::ops::{Add, Mul};
use thiserror::Error;

#[derive(Clone)]
pub struct MachineState<T> {
    memory: Vec<T>,
    pc: usize,
}

pub mod num {

    pub struct NumImpl<T, F>
    where
        F: Fn(T) -> Option<usize>,
    {
        pub to_usize: F,
        pub zero: T,
        pub one: T,
    }
    pub fn i32() -> NumImpl<i32, impl Fn(i32) -> Option<usize>> {
        NumImpl {
            to_usize: |x| if x < 0 { None } else { Some(x as usize) },
            zero: 0,
            one: 1,
        }
    }

    pub fn usize() -> NumImpl<usize, impl Fn(usize) -> Option<usize>> {
        NumImpl {
            to_usize: |x| Some(x),
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

    pub fn new_with_memory<J>(mem: &J) -> MachineState<T>
    where
        J: IntoIterator<Item = T>,
        J: Clone,
    {
        MachineState {
            memory: mem.clone().into_iter().collect(),
            pc: 0,
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
        Ok(StepResult::Stepped)
    }

    pub fn one_step<G>(
        &mut self,
        num: &num::NumImpl<T, G>,
    ) -> Result<StepResult<T>, MachineExecutionError>
    where
        T: Add<T, Output = T> + Mul<T, Output = T> + Copy + std::cmp::Ord,
        G: Fn(T) -> Option<usize>,
    {
        let opcode = *self.read_mem_elt(self.pc)?;
        let opcode: usize = (num.to_usize)(opcode).ok_or(MachineExecutionError::OutOfBounds(
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
                self.process_binary_op(mode_1, mode_2, |a, b| a + b, &num.to_usize)
            }
            2 => {
                if opcode >= 10000 {
                    return Err(MachineExecutionError::BadParameterMode(opcode));
                }
                let mode_1 = ParameterMode::of_int((opcode / 100) % 10)
                    .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
                let mode_2 = ParameterMode::of_int((opcode / 1000) % 10)
                    .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
                self.process_binary_op(mode_1, mode_2, |a, b| a * b, &num.to_usize)
            }
            3 => {
                if opcode != 3 {
                    return Err(MachineExecutionError::BadParameterMode(opcode));
                }
                let location = self.read_mem_elt(self.pc + 1)?;
                let location = (num.to_usize)(*location).ok_or(MemoryAccessError::Negative)?;
                self.pc += 2;
                Ok(StepResult::Io(StepIoResult::AwaitingInput(location)))
            }
            4 => {
                if opcode >= 1000 {
                    return Err(MachineExecutionError::BadParameterMode(opcode));
                }
                let mode = ParameterMode::of_int((opcode / 100) % 10)
                    .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
                let to_output = match mode {
                    ParameterMode::Position => {
                        let addr = (num.to_usize)(*self.read_mem_elt(self.pc + 1)?)
                            .ok_or(MemoryAccessError::Negative)?;
                        *self.read_mem_elt(addr)?
                    }
                    ParameterMode::Immediate => *self.read_mem_elt(self.pc + 1)?,
                };
                self.pc += 2;
                Ok(StepResult::Io(StepIoResult::Output(to_output)))
            }
            5 => {
                if opcode >= 10000 {
                    return Err(MachineExecutionError::BadParameterMode(opcode));
                }
                let mode_comparand = ParameterMode::of_int((opcode / 100) % 10)
                    .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
                let comparand = *self.read_param(self.pc + 1, mode_comparand, &num.to_usize)?;
                if comparand != num.zero {
                    let mode_target = ParameterMode::of_int((opcode / 1000) % 10)
                        .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
                    let target = *self.read_param(self.pc + 2, mode_target, &num.to_usize)?;
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
                let comparand = *self.read_param(self.pc + 1, mode_comparand, &num.to_usize)?;
                if comparand == num.zero {
                    let mode_target = ParameterMode::of_int((opcode / 1000) % 10)
                        .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
                    let target = *self.read_param(self.pc + 2, mode_target, &num.to_usize)?;
                    self.pc = (num.to_usize)(target).ok_or(MemoryAccessError::Negative)?;
                } else {
                    self.pc += 3;
                }
                Ok(StepResult::Stepped)
            }
            7 => {
                if opcode >= 100000 {
                    return Err(MachineExecutionError::BadParameterMode(opcode));
                }
                let mode_1 = ParameterMode::of_int((opcode / 100) % 10)
                    .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
                let mode_2 = ParameterMode::of_int((opcode / 1000) % 10)
                    .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
                self.process_binary_op(
                    mode_1,
                    mode_2,
                    |a, b| if a < b { num.one } else { num.zero },
                    &num.to_usize,
                )?;
                Ok(StepResult::Stepped)
            }
            8 => {
                if opcode >= 100000 {
                    return Err(MachineExecutionError::BadParameterMode(opcode));
                }
                let mode_1 = ParameterMode::of_int((opcode / 100) % 10)
                    .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
                let mode_2 = ParameterMode::of_int((opcode / 1000) % 10)
                    .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
                self.process_binary_op(
                    mode_1,
                    mode_2,
                    |a, b| if a == b { num.one } else { num.zero },
                    &num.to_usize,
                )?;
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

    pub fn execute_until_input<G>(
        &mut self,
        num: &num::NumImpl<T, G>,
    ) -> Result<StepIoResult<T>, MachineExecutionError>
    where
        T: Add<T, Output = T> + Mul<T, Output = T> + Copy + Ord,
        G: Fn(T) -> Option<usize>,
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

    pub fn execute_to_end<G, I>(
        &mut self,
        get_input: &mut I,
        num: &num::NumImpl<T, G>,
    ) -> Result<Vec<T>, MachineExecutionError>
    where
        T: Add<T, Output = T> + Mul<T, Output = T> + Copy + Ord,
        I: Iterator<Item = T>,
        G: Fn(T) -> Option<usize>,
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
        input: &mut I,
        expected_output: &[T],
        num: &num::NumImpl<T, G>,
    ) where
        T: Add<T, Output = T> + Mul<T, Output = T> + Copy + Ord,
        I: Iterator<Item = T>,
        G: Fn(T) -> Option<usize>,
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
}
