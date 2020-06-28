use std::collections::HashMap;
use std::io;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::str::FromStr;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;


/// Extract compilation vars from the specified interpreter.
fn get_config_from_interpreter(interpreter: &Path) -> Result<InterpreterConfig> {
    let script = r#"
import json
import platform
import struct
import sys
import sysconfig

PYPY = platform.python_implementation() == "PyPy"

try:
    base_prefix = sys.base_prefix
except AttributeError:
    base_prefix = sys.exec_prefix

libdir = sysconfig.get_config_var('LIBDIR')

print("version_major", sys.version_info[0])
print("version_minor", sys.version_info[1])
print("implementation", platform.python_implementation())
if libdir is not None:
    print("libdir", libdir)
print("ld_version", sysconfig.get_config_var('LDVERSION') or sysconfig.get_config_var('py_version_short'))
print("base_prefix", base_prefix)
print("shared", PYPY or bool(sysconfig.get_config_var('Py_ENABLE_SHARED')))
print("executable", sys.executable)
print("calcsize_pointer", struct.calcsize("P"))
"#;
    let output = run_python_script(interpreter, script)?;
    let map: HashMap<String, String> = output
        .lines()
        .filter_map(|line| {
            let mut i = line.splitn(2, ' ');
            Some((i.next()?.into(), i.next()?.into()))
        })
        .collect();
    Ok(InterpreterConfig {
        version: PythonVersion {
            major: map["version_major"].parse()?,
            minor: map["version_minor"].parse()?,
            implementation: map["implementation"].parse()?,
        },
        libdir: map.get("libdir").cloned(),
        shared: map["shared"] == "True",
        ld_version: map["ld_version"].clone(),
        base_prefix: map["base_prefix"].clone(),
        executable: map["executable"].clone().into(),
        calcsize_pointer: map["calcsize_pointer"].parse()?,
    })
}

/// Information returned from python interpreter
#[derive(Debug)]
pub struct InterpreterConfig {
    pub version: PythonVersion,
    pub libdir: Option<String>,
    pub shared: bool,
    pub ld_version: String,
    /// Prefix used for determining the directory of libpython
    pub base_prefix: String,
    pub executable: PathBuf,
    pub calcsize_pointer: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum PythonImplementation {
    CPython,
    PyPy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonVersion {
    pub major: u8,
    // minor == None means any minor version will do
    pub minor: u8,
    pub implementation: PythonImplementation,
}

impl fmt::Display for PythonVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} {}.{}", self.implementation, self.major, self.minor)
    }
}

impl FromStr for PythonImplementation {
    type Err = Box<dyn std::error::Error>;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "CPython" => Ok(PythonImplementation::CPython),
            "PyPy" => Ok(PythonImplementation::PyPy),
            _ => Err(format!("Invalid interpreter: {}", s).into()),
        }
    }
}

/// Run a python script using the specified interpreter binary.
fn run_python_script(interpreter: &Path, script: &str) -> Result<String> {
    let out = Command::new(interpreter)
        .args(&["-c", script])
        .stderr(Stdio::inherit())
        .output();

    match out {
        Err(err) => {
            if err.kind() == io::ErrorKind::NotFound {
                return Err(format!(
                    "Could not find any interpreter at {}, \
                     are you sure you have Python installed on your PATH?",
                    interpreter.display()
                )
                .into());
            } else {
                return Err(format!(
                    "Failed to run the Python interpreter at {}: {}",
                    interpreter.display(),
                    err
                )
                .into());
            }
        }
        Ok(ref ok) if !ok.status.success() => {
            return Err(format!("Python script failed: {}", script).into())
        }
        Ok(ok) => Ok(String::from_utf8(ok.stdout)?),
    }
}

/// Search for python interpreters and yield them in order.
///
/// The following locations are checked in the order listed:
///
/// 1. `python`
/// 2. `python3`
pub fn find_interpreters() -> impl Iterator<Item = InterpreterConfig> {
    ["python", "python3"]
        .iter()
        .filter_map(|interpreter| {
            get_config_from_interpreter(Path::new(interpreter)).ok()
        })
}
