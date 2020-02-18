use std::io;
mod lib;
use lib::{read_program, Error, Machine};

fn main() -> Result<(), Error> {
    let program_name = "examples/cat.bfk";

    let mut machine =
        Machine::<_, _, u128>::new(read_program(program_name)?, io::stdin(), io::stdout());
    loop {
        match machine.step() {
            Ok(_) => {}
            Err(Error::ProgramComplete) => break,
            Err(e) => return Err(e),
        };
    }
    Ok(())
}
