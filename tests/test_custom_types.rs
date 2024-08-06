use std::collections::HashMap;

use pyo3::{
    exceptions::{PyIndexError, PyKeyError},
    prelude::*,
    types::{PyDict, PyMapping, PySequence, PyTuple},
};
use pythonize::{
    depythonize, pythonize_custom, PythonizeListType, PythonizeMappingType, PythonizeTypes,
    PythonizeUnnamedMappingAdapter, Pythonizer,
};
use serde::Serialize;
use serde_json::{json, Value};

#[pyclass(sequence)]
struct CustomList {
    items: Vec<PyObject>,
}

#[pymethods]
impl CustomList {
    fn __len__(&self) -> usize {
        self.items.len()
    }

    fn __getitem__(&self, idx: isize) -> PyResult<PyObject> {
        self.items
            .get(idx as usize)
            .cloned()
            .ok_or_else(|| PyIndexError::new_err(idx))
    }
}

impl PythonizeListType for CustomList {
    fn create_sequence<T, U>(
        py: Python,
        elements: impl IntoIterator<Item = T, IntoIter = U>,
    ) -> PyResult<Bound<PySequence>>
    where
        T: ToPyObject,
        U: ExactSizeIterator<Item = T>,
    {
        let sequence = Bound::new(
            py,
            CustomList {
                items: elements
                    .into_iter()
                    .map(|item| item.to_object(py))
                    .collect(),
            },
        )?
        .into_any();

        Ok(unsafe { sequence.downcast_into_unchecked() })
    }
}

struct PythonizeCustomList;
impl<'py> PythonizeTypes<'py> for PythonizeCustomList {
    type Map = PyDict;
    type NamedMap = PythonizeUnnamedMappingAdapter<'py, PyDict>;
    type List = CustomList;
}

#[test]
fn test_custom_list() {
    Python::with_gil(|py| {
        PySequence::register::<CustomList>(py).unwrap();
        let serialized = pythonize_custom::<PythonizeCustomList, _>(py, &json!([1, 2, 3])).unwrap();
        assert!(serialized.is_instance_of::<CustomList>());

        let deserialized: Value = depythonize(&serialized).unwrap();
        assert_eq!(deserialized, json!([1, 2, 3]));
    })
}

#[pyclass(mapping)]
struct CustomDict {
    items: HashMap<String, PyObject>,
}

#[pymethods]
impl CustomDict {
    fn __len__(&self) -> usize {
        self.items.len()
    }

    fn __getitem__(&self, key: String) -> PyResult<PyObject> {
        self.items
            .get(&key)
            .cloned()
            .ok_or_else(|| PyKeyError::new_err(key))
    }

    fn __setitem__(&mut self, key: String, value: PyObject) {
        self.items.insert(key, value);
    }

    fn keys(&self) -> Vec<&String> {
        self.items.keys().collect()
    }

    fn values(&self) -> Vec<PyObject> {
        self.items.values().cloned().collect()
    }
}

impl<'py> PythonizeMappingType<'py> for CustomDict {
    type Builder = Bound<'py, CustomDict>;

    fn builder(py: Python<'py>, len: Option<usize>) -> PyResult<Self::Builder> {
        Bound::new(
            py,
            CustomDict {
                items: HashMap::with_capacity(len.unwrap_or(0)),
            },
        )
    }

    fn push_item(
        builder: &mut Self::Builder,
        key: Bound<'py, PyAny>,
        value: Bound<'py, PyAny>,
    ) -> PyResult<()> {
        unsafe { builder.downcast_unchecked::<PyMapping>() }.set_item(key, value)
    }

    fn finish(builder: Self::Builder) -> PyResult<Bound<'py, PyMapping>> {
        Ok(unsafe { builder.into_any().downcast_into_unchecked() })
    }
}

struct PythonizeCustomDict;
impl<'py> PythonizeTypes<'py> for PythonizeCustomDict {
    type Map = CustomDict;
    type NamedMap = PythonizeUnnamedMappingAdapter<'py, CustomDict>;
    type List = PyTuple;
}

#[test]
fn test_custom_dict() {
    Python::with_gil(|py| {
        PyMapping::register::<CustomDict>(py).unwrap();
        let serialized =
            pythonize_custom::<PythonizeCustomDict, _>(py, &json!({ "hello": 1, "world": 2 }))
                .unwrap();
        assert!(serialized.is_instance_of::<CustomDict>());

        let deserialized: Value = depythonize(&serialized).unwrap();
        assert_eq!(deserialized, json!({ "hello": 1, "world": 2 }));
    })
}

#[test]
fn test_tuple() {
    Python::with_gil(|py| {
        PyMapping::register::<CustomDict>(py).unwrap();
        let serialized =
            pythonize_custom::<PythonizeCustomDict, _>(py, &json!([1, 2, 3, 4])).unwrap();
        assert!(serialized.is_instance_of::<PyTuple>());

        let deserialized: Value = depythonize(&serialized).unwrap();
        assert_eq!(deserialized, json!([1, 2, 3, 4]));
    })
}

#[test]
fn test_pythonizer_can_be_created() {
    // https://github.com/davidhewitt/pythonize/pull/56
    Python::with_gil(|py| {
        let sample = json!({ "hello": 1, "world": 2 });
        assert!(sample
            .serialize(Pythonizer::new(py))
            .unwrap()
            .is_instance_of::<PyDict>());

        assert!(sample
            .serialize(Pythonizer::custom::<PythonizeCustomDict>(py))
            .unwrap()
            .is_instance_of::<CustomDict>());
    })
}
