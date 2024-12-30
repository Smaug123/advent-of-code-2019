use std::num::NonZero;
use std::ops::{Add, Mul};
use thiserror::Error;

#[derive(Clone)]
struct UsizeHashmap<T> {
    indices: Vec<NonZero<usize>>,
    contents: Vec<T>,
}

impl<T> UsizeHashmap<T> {
    // Look up the index at which `i` appears, if it does.
    fn find_index(&self, i: NonZero<usize>) -> Option<usize> {
        self.indices
            .iter()
            .copied()
            .enumerate()
            .find(|(_vec_index, stored)| i == *stored)
            .map(|(vec_index, _)| vec_index)
    }

    fn find(&self, i: NonZero<usize>) -> Option<T>
    where
        T: Clone,
    {
        self.find_index(i)
            .map(|index| (unsafe { self.contents.get_unchecked(index) }).clone())
    }

    fn insert(&mut self, index: NonZero<usize>, v: T) {
        match self.find_index(index) {
            Some(vec_index) => {
                unsafe {
                    *self.contents.get_unchecked_mut(vec_index) = v;
                };
            }
            None => {
                self.contents.push(v);
                self.indices.push(index);
            }
        }
    }

    fn clear(&mut self) {
        self.indices.clear();
        self.contents.clear();
    }

    fn new() -> Self {
        Self {
            indices: vec![],
            contents: vec![],
        }
    }
}

#[derive(Clone)]
pub struct MachineState<T> {
    memory: Vec<T>,
    sparse_memory: UsizeHashmap<T>,
    pc: usize,
}

pub trait Num {
    fn zero() -> Self;
    fn one() -> Self;
    fn to_usize(self) -> Option<usize>;
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
}

#[derive(Error, Debug)]
pub enum MachineExecutionError {
    #[error("instruction {0} at position {1} unrecognised")]
    BadOpcode(usize, usize),
    #[error("attempted to access negative memory index")]
    OutOfBounds,
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

pub enum MemoryReadResult<'a, T> {
    Dense(T),
    Sparse(&'a T),
    Uninitialised,
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

impl<T> MachineState<T> {
    pub fn new_with_memory<J>(mem: &J) -> MachineState<T>
    where
        J: IntoIterator<Item = T>,
        J: Clone,
        T: Num,
    {
        let mut mem: Vec<T> = mem.clone().into_iter().collect();
        // Critical for safety: need the UsizeHashmap to never contain 0
        if mem.is_empty() {
            mem.push(T::zero())
        }

        MachineState {
            memory: mem,
            sparse_memory: UsizeHashmap::new(),
            pc: 0,
        }
    }

    pub fn reset<J>(&mut self, mem: J)
    where
        J: IntoIterator<Item = T> + Clone,
        T: Num,
    {
        self.pc = 0;
        self.memory.clear();
        self.memory.extend(mem);
        if self.memory.is_empty() {
            self.memory.push(T::zero());
        }
        self.sparse_memory.clear();
    }

    pub fn get_value(&self, result: T) -> T {
        result
    }

    fn process_binary_op<G>(
        &mut self,
        opcode: usize,
        process: G,
    ) -> Result<StepResult<T>, MachineExecutionError>
    where
        T: Copy + Num,
        G: Fn(T, T) -> T,
    {
        if opcode >= 10000 {
            return Err(MachineExecutionError::BadParameterMode(opcode));
        }
        let mode_1 = ParameterMode::of_int((opcode / 100) % 10)
            .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
        let mode_2 = ParameterMode::of_int((opcode / 1000) % 10)
            .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
        let arg_1 = self
            .read_param(self.pc + 1, mode_1)
            .map_err(|()| MachineExecutionError::OutOfBounds)?;
        let arg_2 = self
            .read_param(self.pc + 2, mode_2)
            .map_err(|()| MachineExecutionError::OutOfBounds)?;

        let result_pos = T::to_usize(self.get_value(self.read_mem_elt(self.pc + 3)))
            .ok_or(MachineExecutionError::OutOfBounds)?;

        self.set_mem_elt(
            result_pos,
            process(self.get_value(arg_1), self.get_value(arg_2)),
        );
        self.pc += 4;
        Ok(StepResult::Stepped)
    }

    pub fn one_step(&mut self) -> Result<StepResult<T>, MachineExecutionError>
    where
        T: Add<T, Output = T> + Mul<T, Output = T> + Copy + std::cmp::Ord + Num,
    {
        let opcode = T::to_usize(self.get_value(self.read_mem_elt(self.pc)))
            .ok_or(MachineExecutionError::OutOfBounds)?;
        match opcode % 100 {
            1_usize => self.process_binary_op(opcode, |a, b| a + b),
            2 => self.process_binary_op(opcode, |a, b| a * b),
            3 => {
                if opcode != 3 {
                    return Err(MachineExecutionError::BadParameterMode(opcode));
                }
                let location = T::to_usize(self.get_value(self.read_mem_elt(self.pc + 1)))
                    .ok_or(MachineExecutionError::OutOfBounds)?;
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
                        let addr = T::to_usize(self.get_value(self.read_mem_elt(self.pc + 1)))
                            .ok_or(MachineExecutionError::OutOfBounds)?;
                        self.get_value(self.read_mem_elt(addr))
                    }
                    ParameterMode::Immediate => self.get_value(self.read_mem_elt(self.pc + 1)),
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
                let comparand = self
                    .read_param(self.pc + 1, mode_comparand)
                    .map_err(|()| MachineExecutionError::OutOfBounds)?;
                let comparand = self.get_value(comparand);
                if comparand != T::zero() {
                    let mode_target = ParameterMode::of_int((opcode / 1000) % 10)
                        .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
                    let target = self
                        .read_param(self.pc + 2, mode_target)
                        .map_err(|()| MachineExecutionError::OutOfBounds)?;
                    let target = T::to_usize(self.get_value(target))
                        .ok_or(MachineExecutionError::OutOfBounds)?;
                    self.pc = target;
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
                let comparand = self
                    .read_param(self.pc + 1, mode_comparand)
                    .map_err(|()| MachineExecutionError::OutOfBounds)?;
                let comparand_zero = self.get_value(comparand) == T::zero();
                if comparand_zero {
                    let mode_target = ParameterMode::of_int((opcode / 1000) % 10)
                        .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
                    let target = self
                        .read_param(self.pc + 2, mode_target)
                        .map_err(|()| MachineExecutionError::OutOfBounds)?;
                    let target = T::to_usize(self.get_value(target))
                        .ok_or(MachineExecutionError::OutOfBounds)?;
                    self.pc = target;
                } else {
                    self.pc += 3;
                }
                Ok(StepResult::Stepped)
            }
            7 => {
                self.process_binary_op(opcode, |a, b| if a < b { T::one() } else { T::zero() })?;
                Ok(StepResult::Stepped)
            }
            8 => {
                self.process_binary_op(opcode, |a, b| if a == b { T::one() } else { T::zero() })?;
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
        match self.memory.get_mut(i) {
            None => {
                // SAFETY: `memory` never has length 0, so the `get` will always succeed if `i == 0`
                let i = unsafe { NonZero::new_unchecked(i) };
                self.sparse_memory.insert(i, new_val);
            }
            Some(loc) => {
                *loc = new_val;
            }
        }
    }

    #[cold]
    #[inline(never)] // Perf-critical for the dense case that this never be inlined!
                     // SAFETY: Do not call this with `i = 0`; that is undefined behaviour.
    fn read_sparse(&self, i: usize) -> T
    where
        T: Clone + Num,
    {
        let i = unsafe { NonZero::new_unchecked(i) };
        match self.sparse_memory.find(i) {
            Some(entry) => entry.clone(),
            None => T::zero(),
        }
    }

    #[inline]
    pub fn read_mem_elt(&self, i: usize) -> T
    where
        T: Clone + Num,
    {
        match self.memory.get(i) {
            Some(entry) => entry.clone(),
            None => {
                // SAFETY: `memory` never has length 0, so the `get` earlier will always succeed if
                // `i == 0`.
                self.read_sparse(i)
            }
        }
    }

    // Error outcome: we failed to convert the contents of a piece of memory to an address.
    fn read_param(&self, i: usize, mode: ParameterMode) -> Result<T, ()>
    where
        T: Copy + Num,
    {
        match mode {
            ParameterMode::Immediate => Ok(self.read_mem_elt(i)),
            ParameterMode::Position => {
                let pos = T::to_usize(self.read_mem_elt(i)).ok_or(())?;
                Ok(self.read_mem_elt(pos))
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
}
