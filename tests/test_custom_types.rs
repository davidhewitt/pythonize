use std::collections::HashMap;

use pyo3::{
    exceptions::{PyIndexError, PyKeyError},
    prelude::*,
    types::{PyDict, PyList, PyMapping, PySequence},
};
use pythonize::{
    depythonize, pythonize_custom, PythonizeDictType, PythonizeListType, PythonizeTypes,
};
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
    ) -> PyResult<&PySequence>
    where
        T: ToPyObject,
        U: ExactSizeIterator<Item = T>,
    {
        let sequence = Py::new(
            py,
            CustomList {
                items: elements
                    .into_iter()
                    .map(|item| item.to_object(py))
                    .collect(),
            },
        )?
        .into_ref(py);

        Ok(unsafe { PySequence::try_from_unchecked(sequence.as_ref()) })
    }
}

struct PythonizeCustomList;
impl PythonizeTypes for PythonizeCustomList {
    type Map = PyDict;
    type List = CustomList;
}

#[test]
fn test_custom_list() {
    Python::with_gil(|py| {
        PySequence::register::<CustomList>(py).unwrap();
        let serialized = pythonize_custom::<PythonizeCustomList, _>(py, &json!([1, 2, 3]))
            .unwrap()
            .into_ref(py);
        assert!(serialized.is_instance_of::<CustomList>());

        let deserialized: Value = depythonize(serialized).unwrap();
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

impl PythonizeDictType for CustomDict {
    fn create_mapping(py: Python) -> PyResult<&PyMapping> {
        let mapping = Py::new(
            py,
            CustomDict {
                items: HashMap::new(),
            },
        )?
        .into_ref(py);
        Ok(unsafe { PyMapping::try_from_unchecked(mapping.as_ref()) })
    }
}

struct PythonizeCustomDict;
impl PythonizeTypes for PythonizeCustomDict {
    type Map = CustomDict;
    type List = PyList;
}

#[test]
fn test_custom_dict() {
    Python::with_gil(|py| {
        PyMapping::register::<CustomDict>(py).unwrap();
        let serialized =
            pythonize_custom::<PythonizeCustomDict, _>(py, &json!({ "hello": 1, "world": 2 }))
                .unwrap()
                .into_ref(py);
        assert!(serialized.is_instance_of::<CustomDict>());

        let deserialized: Value = depythonize(serialized).unwrap();
        assert_eq!(deserialized, json!({ "hello": 1, "world": 2 }));
    })
}
