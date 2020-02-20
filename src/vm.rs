#[derive(Debug)]
pub enum Error {
    Language(vm::Error),
    Io(std::io::Error),
}

impl From<vm::Error> for Error {
    fn from(e: vm::Error) -> Self {
        Error::Language(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

pub mod vm {
    use num_traits::{
        cast::FromPrimitive,
        identities::{one, zero},
        int::PrimInt,
        ops::wrapping::{WrappingAdd, WrappingSub},
        sign::Unsigned,
    };

    use std::{
        collections::{HashMap, VecDeque},
        io::{Read, Write},
    };

    use crate::interpret::{Instruction, Program};

    #[derive(Debug)]
    pub enum Error {
        UnmatchedOpenBracket(usize),
        UnmatchedCloseBracket(usize),
    }

    #[derive(Debug)]
    pub struct Machine<R, W, I> {
        program: Program,
        instruction_ptr: usize,
        memory: VecDeque<I>,
        data_ptr: usize,
        open_to_close: HashMap<usize, usize>,
        close_to_open: HashMap<usize, usize>,
        input: R,
        output: W,
    }

    impl<R: Read, W: Write, I: PrimInt + WrappingAdd + WrappingSub + FromPrimitive + Unsigned>
        Machine<R, W, I>
    {
        pub fn new(program: Program, input: R, output: W) -> Self {
            use Instruction::*;

            let mut open_to_close = HashMap::new();
            let mut close_to_open = HashMap::new();
            let mut open_stack = Vec::new();

            for (i, instruction) in program.iter().copied().enumerate() {
                if instruction == While {
                    open_stack.push(i);
                }
                if instruction == WhileEnd {
                    if let Some(open_loc) = open_stack.pop() {
                        open_to_close.insert(open_loc, i);
                        close_to_open.insert(i, open_loc);
                    }
                }
            }

            open_to_close.shrink_to_fit();
            close_to_open.shrink_to_fit();

            Self {
                program,
                instruction_ptr: 0,
                memory: VecDeque::from(vec![zero()]),
                data_ptr: 0,
                open_to_close,
                close_to_open,
                input,
                output,
            }
        }

        // Returns true if the machine is finished.
        pub fn step(&mut self) -> Result<bool, super::Error> {
            use Instruction::*;

            let instruction = self.program.get(self.instruction_ptr);

            if instruction.is_none() {
                return Ok(true);
            }
            let instruction = instruction.unwrap();

            match instruction {
                MoveRight => {
                    self.data_ptr += 1;
                    if self.memory.get(self.data_ptr).is_none() {
                        self.memory.push_back(zero());
                    }
                }
                MoveLeft => {
                    if self.data_ptr == 0 {
                        self.memory.push_front(zero());
                    } else {
                        self.data_ptr -= 1;
                    }
                }
                Increment => {
                    let x = self.memory.get_mut(self.data_ptr).unwrap();
                    *x = x.wrapping_add(&one());
                }
                Decrement => {
                    let x = self.memory.get_mut(self.data_ptr).unwrap();
                    *x = x.wrapping_sub(&one());
                }
                Write => {
                    let buf = [match I::from_u16(256) {
                        Some(i) => *self.memory.get(self.data_ptr).unwrap() % i,
                        None => *self.memory.get(self.data_ptr).unwrap(),
                    }
                    .to_u8()
                    .unwrap()];
                    self.output.write_all(&buf)?;
                }
                Read => {
                    let mut buf = [0_u8; 1];
                    let bytes_read = self.input.read(&mut buf)?;
                    let input = if bytes_read > 0 {
                        I::from_u8(buf[0]).unwrap()
                    } else {
                        I::max_value()
                    };
                    *self.memory.get_mut(self.data_ptr).unwrap() = input;
                }
                While => {
                    if *self.memory.get(self.data_ptr).unwrap() == zero() {
                        match self.open_to_close.get(&self.instruction_ptr) {
                            Some(close_loc) => self.instruction_ptr = *close_loc,
                            None => {
                                return Err(Error::UnmatchedOpenBracket(self.instruction_ptr).into())
                            }
                        }
                    }
                }
                WhileEnd => {
                    if *self.memory.get(self.data_ptr).unwrap() != zero() {
                        match self.close_to_open.get(&self.instruction_ptr) {
                            Some(open_loc) => self.instruction_ptr = *open_loc,
                            None => {
                                return Err(
                                    Error::UnmatchedCloseBracket(self.instruction_ptr).into()
                                )
                            }
                        }
                    }
                }
                Comment(_) => {}
            };

            self.instruction_ptr += 1;

            Ok(false)
        }

        pub fn run(mut self) -> Result<u64, super::Error> {
            let mut time = 0;
            loop {
                time += 1;
                match self.step() {
                    Ok(true) => break,
                    Ok(false) => continue,
                    Err(e) => return Err(e),
                }
            }
            Ok(time)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::vm::*;

    #[test]
    fn cat() {
        let input: &[u8] = &[47, 0, 38, 1, 200];
        let mut output = Vec::new();
        let m = Machine::<_, _, u16>::new(",+[-.,+]".into(), input, &mut output);
        assert!(m.run().is_ok());
        assert_eq!(input, output.as_slice());
    }
}
