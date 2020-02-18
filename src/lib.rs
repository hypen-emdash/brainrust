use num_traits::{
    cast::FromPrimitive,
    identities::{one, zero},
    int::PrimInt,
    ops::wrapping::{WrappingAdd, WrappingSub},
    sign::Unsigned,
};
use std::{
    collections::{HashMap, VecDeque},
    convert::From,
    fs, io,
    io::{Read, Write},
};

pub fn read_program(path: &str) -> io::Result<Vec<Instruction>> {
    Ok(fs::read_to_string(path)?
        .chars()
        .map(|c| c.into())
        .collect())
}

#[derive(Debug)]
pub struct Machine<R, W, I> {
    program: Vec<Instruction>,
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
    pub fn new(program: Vec<Instruction>, input: R, output: W) -> Self {
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

    pub fn step(&mut self) -> Result<(), Error> {
        use Instruction::*;

        let instruction = *self
            .program
            .get(self.instruction_ptr)
            .ok_or(Error::ProgramComplete)?;

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
                        None => return Err(Error::UnmatchedOpenBracket(self.instruction_ptr)),
                    }
                }
            }
            WhileEnd => {
                if *self.memory.get(self.data_ptr).unwrap() != zero() {
                    match self.close_to_open.get(&self.instruction_ptr) {
                        Some(open_loc) => self.instruction_ptr = *open_loc,
                        None => return Err(Error::UnmatchedCloseBracket(self.instruction_ptr)),
                    }
                }
            }
            Comment(_) => {}
        };

        self.instruction_ptr += 1;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Instruction {
    MoveRight,
    MoveLeft,
    Increment,
    Decrement,
    Write,
    Read,
    While,
    WhileEnd,
    Comment(char),
}

impl From<char> for Instruction {
    fn from(c: char) -> Self {
        use Instruction::*;
        match c {
            '>' => MoveRight,
            '<' => MoveLeft,
            '+' => Increment,
            '-' => Decrement,
            '.' => Write,
            ',' => Read,
            '[' => While,
            ']' => WhileEnd,
            _ => Comment(c),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    ProgramComplete,
    UnmatchedOpenBracket(usize),
    UnmatchedCloseBracket(usize),
    Io(io::Error),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}
