#![cfg(feature = "arbitrary_precision")]

use pyo3::prelude::*;
use pythonize::{depythonize, pythonize};
use serde_json::Value;

#[test]
fn test_greater_than_u64_max() {
    Python::attach(|py| {
        let json_str = r#"18446744073709551616"#;
        let value: Value = serde_json::from_str(json_str).unwrap();
        let result = pythonize(py, &value).unwrap();
        let number_str = result.str().unwrap().to_string();

        assert!(result.is_instance_of::<pyo3::types::PyInt>());
        assert_eq!(number_str, "18446744073709551616");
    });
}

#[test]
fn test_less_than_i64_min() {
    Python::attach(|py| {
        let json_str = r#"-9223372036854775809"#;
        let value: Value = serde_json::from_str(json_str).unwrap();
        let result = pythonize(py, &value).unwrap();
        let number_str = result.str().unwrap().to_string();

        assert!(result.is_instance_of::<pyo3::types::PyInt>());
        assert_eq!(number_str, "-9223372036854775809");
    });
}

#[test]
fn test_float() {
    Python::attach(|py| {
        let json_str = r#"3.141592653589793238"#;
        let value: Value = serde_json::from_str(json_str).unwrap();
        let result = pythonize(py, &value).unwrap();
        let num: f32 = result.extract().unwrap();

        assert!(result.is_instance_of::<pyo3::types::PyFloat>());
        assert_eq!(num, 3.141592653589793238); // not {'$serde_json::private::Number': ...}
    });
}

#[test]
fn test_int() {
    Python::attach(|py| {
        let json_str = r#"2"#;
        let value: Value = serde_json::from_str(json_str).unwrap();
        let result = pythonize(py, &value).unwrap();
        let num: i32 = result.extract().unwrap();

        assert!(result.is_instance_of::<pyo3::types::PyInt>());
        assert_eq!(num, 2); // not {'$serde_json::private::Number': '2'}
    });
}

#[test]
fn test_serde_error_if_token_empty() {
    let json_str = r#"{"$serde_json::private::Number": ""}"#;
    let result: Result<Value, _> = serde_json::from_str(json_str);

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("EOF while parsing a value"));
}

#[test]
fn test_serde_error_if_token_invalid() {
    let json_str = r#"{"$serde_json::private::Number": 2}"#;
    let result: Result<Value, _> = serde_json::from_str(json_str);

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("invalid type: integer `2`, expected string containing a number"));
}

#[test]
fn test_token_valid() {
    Python::attach(|py| {
        let json_str = r#"{"$serde_json::private::Number": "2"}"#;
        let value: Value = serde_json::from_str(json_str).unwrap();
        let result = pythonize(py, &value).unwrap();
        let num: i32 = result.extract().unwrap();

        assert!(result.is_instance_of::<pyo3::types::PyInt>());
        assert_eq!(num, 2);
    });
}

#[test]
fn test_depythonize_greater_than_u128_max() {
    Python::attach(|py| {
        // u128::MAX + 1
        let py_int = py
            .eval(c"340282366920938463463374607431768211456", None, None)
            .unwrap();
        let value: Value = depythonize(&py_int).unwrap();

        assert!(value.is_number());
        assert_eq!(value.to_string(), "340282366920938463463374607431768211456");
    });
}
