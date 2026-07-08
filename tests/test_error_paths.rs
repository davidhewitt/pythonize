use std::collections::HashMap;

use pyo3::prelude::*;
use pyo3::types::{PyBool, PyBytes, PyInt, PyList, PyString};
use pythonize::{depythonize, pythonize};

// ---------------------------------------------------------------------------
// AC-1: Non-string map key (HashMap<i64, i32>) → pythonize
// BASELINE: Ok — UNEXPECTED: the serializer does NOT reject integer keys;
// Python dicts accept any hashable key natively.  DictKeyNotString is a
// deserializer-side check (de.rs) and is never triggered during pythonize.
// The informational error-message comment below would apply on the de side:
// // assert!(err.to_string().contains("dict keys must have type str"));
// ---------------------------------------------------------------------------

#[test]
fn test_pythonize_i64_key_map_is_ok() {
    Python::attach(|py| {
        let mut map: HashMap<i64, i32> = HashMap::new();
        map.insert(1, 10);
        map.insert(2, 20);
        // BASELINE: Ok — pythonize produces a Python dict with integer keys;
        // no error is returned by the serializer.
        let result = pythonize(py, &map);
        assert!(result.is_ok());
    });
}

// ---------------------------------------------------------------------------
// AC-2: Python str → depythonize::<i64> → Err
// ---------------------------------------------------------------------------

#[test]
fn test_depythonize_py_str_as_i64_is_err() {
    Python::attach(|py| {
        let py_str = PyString::new(py, "hello");
        let result: Result<i64, _> = depythonize(py_str.as_any());
        assert!(result.is_err());
    });
}

// ---------------------------------------------------------------------------
// AC-3: Python bytes → depythonize::<String> → characterise actual behaviour
// BASELINE: bytes successfully deserializes to String via serde_bytes-style
// path — result depends on runtime; see comment below.
// ---------------------------------------------------------------------------

#[test]
fn test_depythonize_py_bytes_as_string() {
    Python::attach(|py| {
        let py_bytes = PyBytes::new(py, b"hello");
        let result: Result<String, _> = depythonize(py_bytes.as_any());
        // BASELINE: Err — bytes is not a str; depythonize returns an error when
        // deserializing bytes as String because the serde string visitor
        // requires UTF-8 text, not a bytes sequence.
        assert!(result.is_err());
    });
}

// ---------------------------------------------------------------------------
// Additional characterisation: Python int → depythonize::<String>
// BASELINE: Err — an integer cannot be coerced to a Rust String by the
// deserializer; it expects a Python str, not int.
// ---------------------------------------------------------------------------

#[test]
fn test_depythonize_py_int_as_string() {
    Python::attach(|py| {
        let py_int = PyInt::new(py, 42i64);
        let result: Result<String, _> = depythonize(py_int.as_any());
        // BASELINE: Err — int is not accepted as String; no silent coercion.
        assert!(result.is_err());
    });
}

// ---------------------------------------------------------------------------
// Additional characterisation: Python None → depythonize::<i64>
// BASELINE: Err — None is not an integer; deserializer returns an error.
// ---------------------------------------------------------------------------

#[test]
fn test_depythonize_py_none_as_i64() {
    Python::attach(|py| {
        let py_none = py.None();
        let result: Result<i64, _> = depythonize(py_none.bind(py));
        // BASELINE: Err — None cannot deserialize as i64.
        assert!(result.is_err());
    });
}

// ---------------------------------------------------------------------------
// Additional characterisation: Python list → depythonize::<String>
// BASELINE: Err — a sequence cannot deserialize as a bare String.
// ---------------------------------------------------------------------------

#[test]
fn test_depythonize_py_list_as_string() {
    Python::attach(|py| {
        let py_list = PyList::new(py, [1i32, 2, 3]).unwrap();
        let result: Result<String, _> = depythonize(py_list.as_any());
        // BASELINE: Err — list cannot deserialize as String.
        assert!(result.is_err());
    });
}

// ---------------------------------------------------------------------------
// Additional characterisation: Python bool → depythonize::<String>
// BASELINE: Err — a Python bool is not a str; deserializer rejects it.
// ---------------------------------------------------------------------------

#[test]
fn test_depythonize_py_bool_as_string() {
    Python::attach(|py| {
        let py_bool = PyBool::new(py, true);
        let result: Result<String, _> = depythonize(py_bool.as_any());
        // BASELINE: Err — bool is not accepted as String.
        assert!(result.is_err());
    });
}

// ---------------------------------------------------------------------------
// Additional characterisation: HashMap<bool, i32> → pythonize
// BASELINE: Ok — UNEXPECTED: same as i64 keys; the serializer converts bool
// to Python bool and uses it as a dict key without error.  Python dicts
// accept boolean keys natively (True/False are hashable).
// ---------------------------------------------------------------------------

#[test]
fn test_pythonize_bool_key_map_is_ok() {
    Python::attach(|py| {
        let mut map: HashMap<bool, i32> = HashMap::new();
        map.insert(true, 1);
        map.insert(false, 0);
        // BASELINE: Ok — pythonize produces a Python dict with bool keys
        // {True: 1, False: 0}; no error is returned by the serializer.
        let result = pythonize(py, &map);
        assert!(result.is_ok());
    });
}
