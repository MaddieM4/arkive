use arkive::*;
use std::io::Result;

fn main() -> Result<()> {
    Ark::scan("./src")?.write("./copy_of_src")?;
    Ok(())
}
