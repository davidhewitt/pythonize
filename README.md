# Pythonize

***WIP: Most functionality of this crate is still stubbed out. Please don't try to use this yet - unless you're interested in submitting PRs to help finish it off :)***

This is an experimental serializer for Rust's serde ecosystem, which can convert Rust objects to Python values and back.

At the moment the Python structures it produces should be _very_ similar to those which are produced by `serde_json`; i.e. calling Python's `json.loads()` on a value encoded by `serde_json` should produce an identical structure to
that which is produced directly by `pythonize`.

## Usage

Pythonize has two public APIs: `pythonize` and `depythonize`.

```
use serde::{Serialize, Deserialize};
use pyo3::{Python, py_run};
use pythonize::pythonize;

#[derive(Serialize, Deserialize)]
struct Sample {
    foo: String,
    bar: Option<usize>
}

Python::with_gil(|py| -> PyResult<()> {
    let sample = Sample {
        foo: "foo".to_string(),
        bar: None
    };

    let obj = pythonize(py, &sample)?;

    println!("{}", obj.as_ref(py).repr());
})

// XXX: depythonize is not yet implemented!
```
