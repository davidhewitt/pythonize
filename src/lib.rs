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
//! use pyo3::{types::PyAnyMethods, Python};
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
//!     assert_eq!("{'foo': 'Foo', 'bar': None}", &format!("{}", obj.repr().unwrap()));
//!
//!     // Python -> Rust
//!     let new_sample: Sample = depythonize(&obj).unwrap();
//!
//!     assert_eq!(new_sample, sample);
//! });
//!
//! ```
mod de;
mod error;
mod ser;

#[allow(deprecated)]
pub use crate::de::depythonize_bound;
pub use crate::de::{depythonize, Depythonizer};
pub use crate::error::{PythonizeError, Result};
pub use crate::ser::{
    pythonize, pythonize_custom, PythonizeDefault, PythonizeListType, PythonizeMappingType,
    PythonizeNamedMappingType, PythonizeTypes, PythonizeUnnamedMappingAdapter, Pythonizer,
};
