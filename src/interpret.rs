use std::{convert::From, fs, io};

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

#[derive(Debug, PartialEq)]
pub struct Program(Vec<Instruction>);

impl Program {
    pub fn get(&self, i: usize) -> Option<Instruction> {
        self.0.get(i).copied()
    }

    pub fn iter(&self) -> std::slice::Iter<Instruction> {
        self.0.iter()
    }
}

impl From<&str> for Program {
    fn from(s: &str) -> Self {
        Self(s.chars().map(|c| c.into()).collect())
    }
}

pub fn read_program(path: &str) -> io::Result<Program> {
    Ok(fs::read_to_string(path)?.as_str().into())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn str_to_program() {
        use Instruction::*;
        assert_eq!(Program(vec![Read, Increment, While, Decrement, Write, Read, Increment, WhileEnd, MoveLeft, MoveRight]), ",+[-.,+]<>".into());
    }
}
