use std::collections::BTreeMap;

use pyo3::{
    prelude::*,
    types::{PyDict, PyList},
};
use pythonize::{PythonizeTypes, PythonizeUnnamedMappingAdapter};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct Root<T> {
    root_key: String,
    root_map: BTreeMap<String, Nested<T>>,
}

impl<'py, T> PythonizeTypes<'py> for Root<T> {
    type Map = PyDict;
    type NamedMap = PythonizeUnnamedMappingAdapter<'py, PyDict>;
    type List = PyList;
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct Nested<T> {
    nested_key: T,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
struct CannotSerialize {}

impl Serialize for CannotSerialize {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Err(serde::ser::Error::custom(
            "something went intentionally wrong",
        ))
    }
}

#[test]
fn test_de_valid() {
    Python::with_gil(|py| {
        let pyroot = PyDict::new(py);
        pyroot.set_item("root_key", "root_value").unwrap();

        let nested = PyDict::new(py);
        let nested_0 = PyDict::new(py);
        nested_0.set_item("nested_key", "nested_value_0").unwrap();
        nested.set_item("nested_0", nested_0).unwrap();
        let nested_1 = PyDict::new(py);
        nested_1.set_item("nested_key", "nested_value_1").unwrap();
        nested.set_item("nested_1", nested_1).unwrap();

        pyroot.set_item("root_map", nested).unwrap();

        let de = &mut pythonize::Depythonizer::from_object(&pyroot);
        let root: Root<String> = serde_path_to_error::deserialize(de).unwrap();

        assert_eq!(
            root,
            Root {
                root_key: String::from("root_value"),
                root_map: BTreeMap::from([
                    (
                        String::from("nested_0"),
                        Nested {
                            nested_key: String::from("nested_value_0")
                        }
                    ),
                    (
                        String::from("nested_1"),
                        Nested {
                            nested_key: String::from("nested_value_1")
                        }
                    )
                ])
            }
        );
    })
}

#[test]
fn test_de_invalid() {
    Python::with_gil(|py| {
        let pyroot = PyDict::new(py);
        pyroot.set_item("root_key", "root_value").unwrap();

        let nested = PyDict::new(py);
        let nested_0 = PyDict::new(py);
        nested_0.set_item("nested_key", "nested_value_0").unwrap();
        nested.set_item("nested_0", nested_0).unwrap();
        let nested_1 = PyDict::new(py);
        nested_1.set_item("nested_key", 1).unwrap();
        nested.set_item("nested_1", nested_1).unwrap();

        pyroot.set_item("root_map", nested).unwrap();

        let de = &mut pythonize::Depythonizer::from_object(&pyroot);
        let err = serde_path_to_error::deserialize::<_, Root<String>>(de).unwrap_err();

        assert_eq!(err.path().to_string(), "root_map.nested_1.nested_key");
        assert_eq!(err.to_string(), "root_map.nested_1.nested_key: unexpected type: 'int' object cannot be converted to 'PyString'");
    })
}

#[test]
fn test_ser_valid() {
    Python::with_gil(|py| {
        let root = Root {
            root_key: String::from("root_value"),
            root_map: BTreeMap::from([
                (
                    String::from("nested_0"),
                    Nested {
                        nested_key: String::from("nested_value_0"),
                    },
                ),
                (
                    String::from("nested_1"),
                    Nested {
                        nested_key: String::from("nested_value_1"),
                    },
                ),
            ]),
        };

        let ser = pythonize::Pythonizer::<Root<String>>::from(py);
        let pyroot: Bound<'_, PyAny> = serde_path_to_error::serialize(&root, ser).unwrap();

        let pyroot = pyroot.downcast::<PyDict>().unwrap();
        assert_eq!(pyroot.len(), 2);

        let root_value: String = pyroot
            .get_item("root_key")
            .unwrap()
            .unwrap()
            .extract()
            .unwrap();
        assert_eq!(root_value, "root_value");

        let root_map = pyroot
            .get_item("root_map")
            .unwrap()
            .unwrap()
            .downcast_into::<PyDict>()
            .unwrap();
        assert_eq!(root_map.len(), 2);

        let nested_0 = root_map
            .get_item("nested_0")
            .unwrap()
            .unwrap()
            .downcast_into::<PyDict>()
            .unwrap();
        assert_eq!(nested_0.len(), 1);
        let nested_key_0: String = nested_0
            .get_item("nested_key")
            .unwrap()
            .unwrap()
            .extract()
            .unwrap();
        assert_eq!(nested_key_0, "nested_value_0");

        let nested_1 = root_map
            .get_item("nested_1")
            .unwrap()
            .unwrap()
            .downcast_into::<PyDict>()
            .unwrap();
        assert_eq!(nested_1.len(), 1);
        let nested_key_1: String = nested_1
            .get_item("nested_key")
            .unwrap()
            .unwrap()
            .extract()
            .unwrap();
        assert_eq!(nested_key_1, "nested_value_1");
    });
}

#[test]
fn test_ser_invalid() {
    Python::with_gil(|py| {
        let root = Root {
            root_key: String::from("root_value"),
            root_map: BTreeMap::from([
                (
                    String::from("nested_0"),
                    Nested {
                        nested_key: CannotSerialize {},
                    },
                ),
                (
                    String::from("nested_1"),
                    Nested {
                        nested_key: CannotSerialize {},
                    },
                ),
            ]),
        };

        let ser = pythonize::Pythonizer::<Root<String>>::from(py);
        let err = serde_path_to_error::serialize(&root, ser).unwrap_err();

        assert_eq!(err.path().to_string(), "root_map.nested_0.nested_key");
    });
}
