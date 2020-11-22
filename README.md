# Pythonize

This is an experimental serializer for Rust's serde ecosystem, which can convert Rust objects to Python values and back.

At the moment the Python structures it produces should be _very_ similar to those which are produced by `serde_json`; i.e. calling Python's `json.loads()` on a value encoded by `serde_json` should produce an identical structure to
that which is produced directly by `pythonize`.

## Usage

This crate converts Rust types which implement the [Serde] serialization
traits into Python objects using the [PyO3] library.

Pythonize has two public APIs: `pythonize` and `depythonize`.

[Serde]: https://github.com/serde-rs/serde
[PyO3]: https://github.com/PyO3/pyo3

# Examples

```rust
use serde::{Serialize, Deserialize};
use pyo3::Python;
use pythonize::{depythonize, pythonize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Sample {
    foo: String,
    bar: Option<usize>
}

let gil = Python::acquire_gil();
let py = gil.python();

let sample = Sample {
    foo: "Foo".to_string(),
    bar: None
};

// Rust -> Python
let obj = pythonize(py, &sample).unwrap();

assert_eq!("{'foo': 'Foo', 'bar': None}", &format!("{}", obj.as_ref(py).repr().unwrap()));

// Python -> Rust
let new_sample: Sample = depythonize(obj.as_ref(py)).unwrap();

assert_eq!(new_sample, sample);
```
