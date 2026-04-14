use pyo3::prelude::*;
use pyo3::types::PyDict;
use pythonize::{depythonize, pythonize};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Simple {
    name: String,
    value: i64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct WithRename {
    #[serde(rename = "firstName")]
    first_name: String,
    age: i32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WithRenameAll {
    first_name: String,
    last_name: String,
    year_of_birth: i32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct WithOption {
    label: String,
    count: Option<i64>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Strict {
    id: i32,
    tag: String,
}

// --- Simple struct ---

#[test]
fn test_simple_round_trip() {
    Python::attach(|py| {
        let original = Simple {
            name: "Alice".into(),
            value: 42,
        };
        let py_obj = pythonize(py, &original).unwrap();
        let result: Simple = depythonize(&py_obj).unwrap();
        assert_eq!(result, original);
    });
}

#[test]
fn test_simple_empty_string_zero() {
    Python::attach(|py| {
        let original = Simple {
            name: String::new(),
            value: 0,
        };
        let py_obj = pythonize(py, &original).unwrap();
        let result: Simple = depythonize(&py_obj).unwrap();
        assert_eq!(result, original);
    });
}

#[test]
fn test_simple_boundary_i64_min() {
    Python::attach(|py| {
        let original = Simple {
            name: "hello world".into(),
            value: i64::MIN,
        };
        let py_obj = pythonize(py, &original).unwrap();
        let result: Simple = depythonize(&py_obj).unwrap();
        assert_eq!(result, original);
    });
}

// --- #[serde(rename)] ---

#[test]
fn test_rename_key_present() {
    Python::attach(|py| {
        let original = WithRename {
            first_name: "Bob".into(),
            age: 30,
        };
        let py_obj = pythonize(py, &original).unwrap();
        let dict = py_obj.cast::<PyDict>().unwrap();
        assert!(dict.get_item("firstName").unwrap().is_some());
        assert!(dict.get_item("first_name").unwrap().is_none());
    });
}

#[test]
fn test_rename_round_trip() {
    Python::attach(|py| {
        let original = WithRename {
            first_name: "Bob".into(),
            age: 30,
        };
        let py_obj = pythonize(py, &original).unwrap();
        let result: WithRename = depythonize(&py_obj).unwrap();
        assert_eq!(result, original);
    });
}

// --- #[serde(rename_all = "camelCase")] ---

#[test]
fn test_rename_all_keys_present() {
    Python::attach(|py| {
        let original = WithRenameAll {
            first_name: "Jane".into(),
            last_name: "Doe".into(),
            year_of_birth: 1990,
        };
        let py_obj = pythonize(py, &original).unwrap();
        let dict = py_obj.cast::<PyDict>().unwrap();
        assert!(dict.get_item("firstName").unwrap().is_some());
        assert!(dict.get_item("lastName").unwrap().is_some());
        assert!(dict.get_item("yearOfBirth").unwrap().is_some());
    });
}

#[test]
fn test_rename_all_round_trip() {
    Python::attach(|py| {
        let original = WithRenameAll {
            first_name: "Jane".into(),
            last_name: "Doe".into(),
            year_of_birth: 1990,
        };
        let py_obj = pythonize(py, &original).unwrap();
        let result: WithRenameAll = depythonize(&py_obj).unwrap();
        assert_eq!(result, original);
    });
}

// --- Unknown fields (default: ignore) ---

#[test]
fn test_unknown_fields_ignored() {
    Python::attach(|py| {
        let dict = PyDict::new(py);
        dict.set_item("name", "test").unwrap();
        dict.set_item("value", 1i64).unwrap();
        dict.set_item("extra", "ignored").unwrap();
        let result: Result<Simple, _> = depythonize(dict.as_any());
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Simple {
                name: "test".into(),
                value: 1
            }
        );
    });
}

// --- #[serde(deny_unknown_fields)] ---

#[test]
fn test_deny_unknown_fields_fails() {
    Python::attach(|py| {
        let dict = PyDict::new(py);
        dict.set_item("id", 1i32).unwrap();
        dict.set_item("tag", "hello").unwrap();
        dict.set_item("extra", "bad").unwrap();
        let result: Result<Strict, _> = depythonize(dict.as_any());
        assert!(result.is_err());
    });
}

// --- Option<T> None ---

#[test]
fn test_option_none_round_trip() {
    Python::attach(|py| {
        let original = WithOption {
            label: "x".into(),
            count: None,
        };
        let py_obj = pythonize(py, &original).unwrap();
        let dict = py_obj.cast::<PyDict>().unwrap();
        let count_opt = dict.get_item("count").unwrap();
        assert!(count_opt.is_some());
        assert!(count_opt.unwrap().is_none());
        let result: WithOption = depythonize(&py_obj).unwrap();
        assert_eq!(result, original);
    });
}

// --- Option<T> Some ---

#[test]
fn test_option_some_round_trip() {
    Python::attach(|py| {
        let original = WithOption {
            label: "y".into(),
            count: Some(99),
        };
        let py_obj = pythonize(py, &original).unwrap();
        let result: WithOption = depythonize(&py_obj).unwrap();
        assert_eq!(result, original);
    });
}

#[test]
fn test_option_some_i64_max() {
    Python::attach(|py| {
        let original = WithOption {
            label: "z".into(),
            count: Some(i64::MAX),
        };
        let py_obj = pythonize(py, &original).unwrap();
        let result: WithOption = depythonize(&py_obj).unwrap();
        assert_eq!(result, original);
    });
}
