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
//! let gil = Python::acquire_gil();
//! let py = gil.python();
//!
//! let sample = Sample {
//!     foo: "Foo".to_string(),
//!     bar: None
//! };
//!
//! // Rust -> Python
//! let obj = pythonize(py, &sample).unwrap();
//!
//! assert_eq!("{'foo': 'Foo', 'bar': None}", &format!("{}", obj.as_ref(py).repr().unwrap()));
//!
//! // Python -> Rust
//! let new_sample: Sample = depythonize(obj.as_ref(py)).unwrap();
//!
//! assert_eq!(new_sample, sample);
//! ```
mod de;
#[cfg(feature = "hashable_dict")]
pub mod dict;
mod error;
mod ser;

pub use crate::de::depythonize;
pub use crate::error::{PythonizeError, Result};
pub use crate::ser::pythonize;

#[doc(hidden)]
#[macro_export]
macro_rules! __cfg_if_hashable_dict {
	($($item:item)*) => {$(
		#[cfg(feature = "hashable_dict")]
		$item
	)*};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __cfg_if_not_hashable_dict {
	($($item:item)*) => {$(
		#[cfg(not(feature = "hashable_dict"))]
		$item
	)*};
}
