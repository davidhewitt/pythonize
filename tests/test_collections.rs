use std::collections::{BTreeMap, HashMap};

use maplit::{btreemap, hashmap};
use pyo3::prelude::*;
use pythonize::{depythonize, pythonize};

fn round_trip<T>(py: Python<'_>, val: T) -> T
where
    T: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug + PartialEq,
{
    let py_val = pythonize(py, &val).expect("pythonize failed");
    depythonize(&py_val).expect("depythonize failed")
}

// --- Vec<i32> ---

#[test]
fn test_vec_i32_empty() {
    Python::attach(|py| {
        let result = round_trip(py, Vec::<i32>::new());
        assert_eq!(result, Vec::<i32>::new());
    });
}

#[test]
fn test_vec_i32_single() {
    Python::attach(|py| {
        assert_eq!(round_trip(py, vec![42i32]), vec![42i32]);
    });
}

#[test]
fn test_vec_i32_three_elements() {
    Python::attach(|py| {
        assert_eq!(round_trip(py, vec![1i32, -7, 100]), vec![1i32, -7, 100]);
    });
}

// --- Vec<f64> ---

#[test]
fn test_vec_f64_three_elements() {
    Python::attach(|py| {
        let input = vec![0.0f64, -1.5, 3.14];
        let result = round_trip(py, input.clone());
        assert_eq!(result, input);
    });
}

// --- Vec<String> ---

#[test]
fn test_vec_string_empty_vec() {
    Python::attach(|py| {
        let result = round_trip(py, Vec::<String>::new());
        assert_eq!(result, Vec::<String>::new());
    });
}

#[test]
fn test_vec_string_empty_strings() {
    Python::attach(|py| {
        let input = vec!["".to_string(), "".to_string()];
        assert_eq!(round_trip(py, input.clone()), input);
    });
}

#[test]
fn test_vec_string_non_empty() {
    Python::attach(|py| {
        let input = vec!["hello".to_string(), "world".to_string()];
        assert_eq!(round_trip(py, input.clone()), input);
    });
}

// --- Vec<bool> ---

#[test]
fn test_vec_bool_mixed() {
    Python::attach(|py| {
        let input = vec![true, false, true, false];
        assert_eq!(round_trip(py, input.clone()), input);
    });
}

// --- Vec<Vec<i32>> ---

#[test]
fn test_vec_nested_two_inner_vecs() {
    Python::attach(|py| {
        let input = vec![vec![1i32, 2, 3], vec![-1i32, 0]];
        assert_eq!(round_trip(py, input.clone()), input);
    });
}

// --- Tuple characterisation ---

#[test]
fn test_tuple_i32_string_round_trip_ok() {
    Python::attach(|py| {
        let input = (7i32, "hello".to_string());
        let py_val = pythonize(py, &input).expect("pythonize failed");
        let result: (i32, String) = depythonize(&py_val).expect("depythonize failed");
        assert_eq!(result.0, 7i32);
        assert_eq!(result.1, "hello");
    });
}

#[test]
fn test_tuple_bool_f64_i64_round_trip_ok() {
    Python::attach(|py| {
        let input = (true, 2.718f64, -42i64);
        let py_val = pythonize(py, &input).expect("pythonize failed");
        let result: (bool, f64, i64) = depythonize(&py_val).expect("depythonize failed");
        assert_eq!(result.0, true);
        assert_eq!(result.1, 2.718f64);
        assert_eq!(result.2, -42i64);
    });
}

// --- HashMap<String, i64> ---

#[test]
fn test_hashmap_string_i64_empty() {
    Python::attach(|py| {
        let input: HashMap<String, i64> = hashmap! {};
        let result = round_trip(py, input);
        assert_eq!(result, HashMap::new());
    });
}

#[test]
fn test_hashmap_string_i64_three_entries() {
    Python::attach(|py| {
        let input: HashMap<String, i64> = hashmap! {
            "a".to_string() => 1,
            "b".to_string() => -2,
            "c".to_string() => 300,
        };
        let result = round_trip(py, input.clone());
        assert_eq!(result, input);
    });
}

#[test]
fn test_hashmap_string_vec_i32_two_entries() {
    Python::attach(|py| {
        let input: HashMap<String, Vec<i32>> = hashmap! {
            "evens".to_string() => vec![2, 4, 6],
            "odds".to_string()  => vec![1, 3, 5],
        };
        let result = round_trip(py, input.clone());
        assert_eq!(result, input);
    });
}

#[test]
fn test_hashmap_string_bool_two_entries() {
    Python::attach(|py| {
        let input: HashMap<String, bool> = hashmap! {
            "yes".to_string() => true,
            "no".to_string()  => false,
        };
        let result = round_trip(py, input.clone());
        assert_eq!(result, input);
    });
}

// --- BTreeMap<String, i64> ---

#[test]
fn test_btreemap_string_i64_empty() {
    Python::attach(|py| {
        let input: BTreeMap<String, i64> = btreemap! {};
        let result = round_trip(py, input);
        assert_eq!(result, BTreeMap::new());
    });
}

#[test]
fn test_btreemap_string_i64_three_entries() {
    Python::attach(|py| {
        let input: BTreeMap<String, i64> = btreemap! {
            "alpha".to_string()   => 10,
            "beta".to_string()    => -20,
            "gamma".to_string()   => 300,
        };
        let result = round_trip(py, input.clone());
        assert_eq!(result, input);
    });
}

#[test]
fn test_btreemap_string_string_two_entries() {
    Python::attach(|py| {
        let input: BTreeMap<String, String> = btreemap! {
            "key1".to_string() => "value1".to_string(),
            "key2".to_string() => "value2".to_string(),
        };
        let result = round_trip(py, input.clone());
        assert_eq!(result, input);
    });
}
