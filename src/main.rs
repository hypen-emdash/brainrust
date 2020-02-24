use std::io;

mod interpret;
mod vm;

use interpret::read_program;
use vm::{vm::Machine, Error};

fn main() -> Result<(), Error> {
    let program_name = "examples/cat.bfk";

    let machine =
        Machine::<_, _, u128>::new(read_program(program_name)?, io::stdin(), io::stdout());
    machine.run()?;
    Ok(())
}
