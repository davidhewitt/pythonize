#![doc = include_str!("../README.md")]

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
