# python-config

** This crate is no longer actively maintained. For similar functionality, see [`pyo3-build-config`]. **

This crate contains functionality to locate and get configuration information about Python interpreters.

Typical usage may be to use this crate in build scripts to search for a Python interpreter:

```
use python_config::find_interpreter_matching;

fn main() {
    let interpreter_config = find_interpreter_matching(|c| c.version.major >= 3)
        .expect("Could not find Python 3 interpreter");

    // Use interpreter_config to configure your package.
}

```

See `examples/print_info` for a more complete demonstration how to use this crate.

## Contributing

At the moment this library is very barebones; if you would find this functionality useful all PRs are welcome to extend this package to a more complete form.

[`pyo3-build-config`]: https://github.com/PyO3/pyo3/tree/main/pyo3-build-config
