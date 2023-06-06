//! This crate converts Rust types which implement the [Serde] serialization
//! traits into Python objects using the [PyO3] library.
//!
//! Pythonize has two public APIs: `pythonize` and `depythonize`.
//!
//! [Serde]: https://github.com/serde-rs/serde
//! [PyO3]: https://github.com/PyO3/pyo3
//!
//! # Examples
//! ```
//! use serde::{Serialize, Deserialize};
//! use pyo3::Python;
//! use pythonize::{depythonize, pythonize};
//!
//! #[derive(Debug, Serialize, Deserialize, PartialEq)]
//! struct Sample {
//!     foo: String,
//!     bar: Option<usize>
//! }
//!
//! Python::with_gil(|py| {
//!     let sample = Sample {
//!         foo: "Foo".to_string(),
//!         bar: None
//!     };
//!
//!     // Rust -> Python
//!     let obj = pythonize(py, &sample).unwrap();
//!
//!     assert_eq!("{'foo': 'Foo', 'bar': None}", &format!("{}", obj.as_ref(py).repr().unwrap()));
//!
//!     // Python -> Rust
//!     let new_sample: Sample = depythonize(obj.as_ref(py)).unwrap();
//!
//!     assert_eq!(new_sample, sample);
//! });
//!
//! ```
mod de;
mod error;
mod ser;

pub use crate::de::depythonize;
pub use crate::error::{PythonizeError, Result};
pub use crate::ser::{
    pythonize, pythonize_custom, PythonizeDictType, PythonizeListType, PythonizeTypes,
};
