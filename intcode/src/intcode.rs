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
            sparse_memory: HashMap::new(),
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
            sparse_memory: HashMap::new(),
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
        self.sparse_memory.clear();
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
            .map_err(|()| MachineExecutionError::OutOfBounds)?.copied()
            .unwrap_or(T::zero());
        let arg_2 = self
            .read_param(self.pc + 2, mode_2)
            .map_err(|()| MachineExecutionError::OutOfBounds)?.copied()
            .unwrap_or(T::zero());

        let result_pos = match self.read_mem_elt(self.pc + 3) {
            None => 0,
            Some(result_pos) => {
                T::to_usize(*result_pos).ok_or(MachineExecutionError::OutOfBounds)?
            }
        };

        self.set_mem_elt(result_pos, process(arg_1, arg_2));
        self.pc += 4;
        Ok(StepResult::Stepped)
    }

    pub fn one_step(&mut self) -> Result<StepResult<T>, MachineExecutionError>
    where
        T: Add<T, Output = T> + Mul<T, Output = T> + Copy + std::cmp::Ord + Num,
    {
        let opcode = match self.read_mem_elt(self.pc) {
            None => 0,
            Some(opcode) => T::to_usize(*opcode).ok_or(MachineExecutionError::OutOfBounds)?,
        };
        match opcode % 100 {
            1_usize => self.process_binary_op(opcode, |a, b| a + b),
            2 => self.process_binary_op(opcode, |a, b| a * b),
            3 => {
                if opcode != 3 {
                    return Err(MachineExecutionError::BadParameterMode(opcode));
                }
                let location = match self.read_mem_elt(self.pc + 1) {
                    None => 0,
                    Some(location) => {
                        T::to_usize(*location).ok_or(MachineExecutionError::OutOfBounds)?
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
                let to_output = match mode {
                    ParameterMode::Position => {
                        let addr = match self.read_mem_elt(self.pc + 1) {
                            None => 0,
                            Some(addr) => {
                                T::to_usize(*addr).ok_or(MachineExecutionError::OutOfBounds)?
                            }
                        };
                        self.read_mem_elt(addr).copied()
                    }
                    ParameterMode::Immediate => self.read_mem_elt(self.pc + 1).copied(),
                };
                self.pc += 2;
                Ok(StepResult::Io(StepIoResult::Output(
                    to_output.unwrap_or(T::zero()),
                )))
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
                match comparand {
                    None => {
                        self.pc += 3;
                    }
                    Some(comparand) => {
                        if *comparand != T::zero() {
                            let mode_target = ParameterMode::of_int((opcode / 1000) % 10)
                                .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
                            let target = self
                                .read_param(self.pc + 2, mode_target)
                                .map_err(|()| MachineExecutionError::OutOfBounds)?
                                .map(|x| T::to_usize(*x).ok_or(MachineExecutionError::OutOfBounds))
                                .unwrap_or(Ok(0))?;
                            self.pc = target;
                        } else {
                            self.pc += 3;
                        }
                    }
                }
                Ok(StepResult::Stepped)
            }
            6 => {
                if opcode >= 10000 {
                    return Err(MachineExecutionError::BadParameterMode(opcode));
                }
                let mode_comparand = ParameterMode::of_int((opcode / 100) % 10)
                    .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
                let comparand_zero = self
                    .read_param(self.pc + 1, mode_comparand)
                    .map_err(|()| MachineExecutionError::OutOfBounds)?
                    .map(|x| *x == T::zero())
                    .unwrap_or(true);
                if comparand_zero {
                    let mode_target = ParameterMode::of_int((opcode / 1000) % 10)
                        .ok_or(MachineExecutionError::BadParameterMode(opcode))?;
                    let target = self
                        .read_param(self.pc + 2, mode_target)
                        .map_err(|()| MachineExecutionError::OutOfBounds)?
                        .map(|x| T::to_usize(*x).ok_or(MachineExecutionError::OutOfBounds))
                        .unwrap_or(Ok(0))?;
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
                self.sparse_memory.insert(i, new_val);
            }
            Some(loc) => {
                *loc = new_val;
            }
        }
    }

    // All outcomes are "success". A None result means the memory is not
    // yet initialised (so is implicitly 0), and is outside the dense array storage.
    pub fn read_mem_elt(&self, i: usize) -> Option<&T> {
        match self.memory.get(i) {
            None => self.sparse_memory.get(&i),
            Some(mem) => Some(mem),
        }
    }

    // Success outcome:
    // A None result means the memory is not yet initialised, so is implicitly 0 (and is outside the dense array storage).
    // Error outcome: we failed to convert the contents of a piece of memory to an address.
    fn read_param(&self, i: usize, mode: ParameterMode) -> Result<Option<&T>, ()>
    where
        T: Copy + Num,
    {
        match mode {
            ParameterMode::Immediate => Ok(self.read_mem_elt(i)),
            ParameterMode::Position => {
                let pos = match self.read_mem_elt(i) {
                    None => 0usize,
                    Some(pos) => T::to_usize(*pos).ok_or(())?,
                };
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
