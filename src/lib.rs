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
mod visitor;

#[allow(deprecated)]
pub use crate::de::depythonize;
pub use crate::de::{depythonize_bound, Depythonizer};
pub use crate::error::{PythonizeError, Result};
pub use crate::ser::{
    pythonize, pythonize_custom, PythonizeDefault, PythonizeDictType, PythonizeListType,
    PythonizeTypes, Pythonizer,
};
pub use crate::visitor::PyObjectVisitor;

#[cfg(feature = "serde_with")]
/// This module provides a Serde `Serialize` and `Deserialize` implementation for `PyObject`.
///
/// ```rust
/// use pyo3::PyObject;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct Foo {
///   #[serde(with = "pythonize::pyobject")]
///   #[serde(flatten)]
///   inner: PyObject,
/// }
/// ```
pub mod pyobject {
    use pyo3::{PyObject, Python};
    use serde::Serializer;

    pub fn serialize<S>(obj: &PyObject, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Python::with_gil(|py| {
            let mut deserializer =
                crate::Depythonizer::from_object_bound(obj.clone().into_bound(py));
            serde_transcode::transcode(&mut deserializer, serializer)
        })
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<PyObject, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Python::with_gil(|py| deserializer.deserialize_any(crate::PyObjectVisitor::new(py)))
    }
}
