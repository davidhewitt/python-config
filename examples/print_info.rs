use python_config::{find_interpreters, InterpreterConfig, Result};

fn find_interpreter() -> Result<InterpreterConfig> {
    for interpreter in find_interpreters() {
        if interpreter.version.major == 3 {
            return Ok(interpreter);
        }
    }

    Err("No Python 3.x interpreter found".into())
}

fn main() -> Result<()> {
    let config = find_interpreter()?;

    println!("interpreter version: {}", config.version);
    println!("interpreter path: {}", config.executable.display());
    println!("libdir: {:?}", config.libdir);
    println!("shared: {}", config.shared);
    println!("base prefix: {}", config.base_prefix);
    println!("ld_version: {}", config.ld_version);
    println!("pointer width: {}", config.calcsize_pointer);

    Ok(())
}
