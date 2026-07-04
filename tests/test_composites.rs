use std::collections::HashMap;

use pyo3::prelude::*;
use pythonize::{depythonize, pythonize};
use serde::{Deserialize, Serialize};

// FR-013: Color enum at module scope — shared across three test functions
// (one exception to the "inside function" rule).
#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum Color {
    Red,                    // unit variant
    Rgb(u8, u8, u8),        // tuple variant
    Named { name: String }, // struct variant
}

// ---------------------------------------------------------------------------
// FR-008 — Option<Vec<i64>> round-trip
// ---------------------------------------------------------------------------

#[test]
fn test_option_vec_none_round_trip() {
    Python::attach(|py| {
        let val: Option<Vec<i64>> = None;
        let result = pythonize(py, &val).and_then(|o| depythonize::<Option<Vec<i64>>>(&o));
        // BASELINE: None round-trips as None.
        assert_eq!(result.unwrap(), val);
    });
}

#[test]
fn test_option_vec_some_empty_round_trip() {
    Python::attach(|py| {
        let val: Option<Vec<i64>> = Some(vec![]);
        let result = pythonize(py, &val).and_then(|o| depythonize::<Option<Vec<i64>>>(&o));
        // BASELINE: Some(vec![]) round-trips as Some(vec![]).
        assert_eq!(result.unwrap(), val);
    });
}

#[test]
fn test_option_vec_some_values_round_trip() {
    Python::attach(|py| {
        let val: Option<Vec<i64>> = Some(vec![1, 2, 3]);
        let result = pythonize(py, &val).and_then(|o| depythonize::<Option<Vec<i64>>>(&o));
        // BASELINE: Some([1, 2, 3]) round-trips intact.
        assert_eq!(result.unwrap(), val);
    });
}

// ---------------------------------------------------------------------------
// FR-009 — Vec<Option<i64>> round-trip
// ---------------------------------------------------------------------------

#[test]
fn test_vec_option_with_nones_round_trip() {
    Python::attach(|py| {
        let val: Vec<Option<i64>> = vec![Some(1i64), None, Some(3)];
        let result = pythonize(py, &val).and_then(|o| depythonize::<Vec<Option<i64>>>(&o));
        // BASELINE: None entries are preserved at their indices; no index shift.
        assert_eq!(result.unwrap(), val);
    });
}

#[test]
fn test_vec_option_all_nones_round_trip() {
    Python::attach(|py| {
        let val: Vec<Option<i64>> = vec![None::<i64>, None, None];
        let result = pythonize(py, &val).and_then(|o| depythonize::<Vec<Option<i64>>>(&o));
        // BASELINE: all-None vec round-trips as three None entries.
        assert_eq!(result.unwrap(), val);
    });
}

// ---------------------------------------------------------------------------
// FR-010 — HashMap<String, Option<i64>> null value semantics
// ---------------------------------------------------------------------------

#[test]
fn test_hashmap_string_option_none_value_round_trip() {
    Python::attach(|py| {
        let mut val: HashMap<String, Option<i64>> = HashMap::new();
        val.insert("key".to_string(), None::<i64>);
        val.insert("other".to_string(), Some(42i64));
        let result =
            pythonize(py, &val).and_then(|o| depythonize::<HashMap<String, Option<i64>>>(&o));
        // FR-010: characterising whether None dict value survives round-trip.
        // BASELINE: None value is preserved under "key"; not dropped or transmuted.
        assert_eq!(result.unwrap(), val);
    });
}

#[test]
fn test_struct_option_field_explicit_none_round_trip() {
    Python::attach(|py| {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct MaybeVal {
            name: String,
            count: Option<i64>,
        }

        let val = MaybeVal {
            name: "test".to_string(),
            count: None,
        };
        let result = pythonize(py, &val).and_then(|o| depythonize::<MaybeVal>(&o));
        // FR-010: explicit None field vs missing key — asserting observed behaviour.
        // BASELINE: count: None round-trips; pythonize emits explicit null, not absent key.
        assert_eq!(result.unwrap(), val);
    });
}

// ---------------------------------------------------------------------------
// FR-012 — Newtype struct and 2-tuple struct round-trip
// ---------------------------------------------------------------------------

#[test]
fn test_newtype_struct_round_trip() {
    Python::attach(|py| {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct Wrapper(i64);

        let val = Wrapper(42i64);
        let result = pythonize(py, &val).and_then(|o| depythonize::<Wrapper>(&o));
        // BASELINE: newtype struct serialises as its inner value (42);
        // depythonize reconstructs Wrapper(42) from the raw integer.
        assert_eq!(result.unwrap(), val);
    });
}

#[test]
fn test_tuple_struct_round_trip() {
    Python::attach(|py| {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct Pair(i64, String);

        let val = Pair(7i64, String::from("hello"));
        let result = pythonize(py, &val).and_then(|o| depythonize::<Pair>(&o));
        // BASELINE: tuple struct serialises as a Python list [7, "hello"];
        // depythonize reconstructs Pair(7, "hello") from the list.
        assert_eq!(result.unwrap(), val);
    });
}

// ---------------------------------------------------------------------------
// FR-013 — Enum variant representations
// ---------------------------------------------------------------------------

#[test]
fn test_enum_unit_variant_round_trip() {
    Python::attach(|py| {
        let val = Color::Red;
        let py_obj = pythonize(py, &val).expect("pythonize Color::Red failed");
        let repr = py_obj.repr().unwrap().to_string();
        eprintln!("FR-013 unit variant repr: {repr}");
        let result: Result<Color, _> = depythonize(&py_obj);
        // BASELINE: Color::Red serialises as the Python string 'Red' (observed repr: 'Red').
        assert_eq!(result.unwrap(), val);
    });
}

#[test]
fn test_enum_tuple_variant_round_trip() {
    Python::attach(|py| {
        let val = Color::Rgb(255, 128, 0);
        let py_obj = pythonize(py, &val).expect("pythonize Color::Rgb failed");
        let repr = py_obj.repr().unwrap().to_string();
        eprintln!("FR-013 tuple variant repr: {repr}");
        let result: Result<Color, _> = depythonize(&py_obj);
        // BASELINE: Color::Rgb(255,128,0) serialises as {'Rgb': (255, 128, 0)} —
        // value is a Python *tuple*, not a list (observed repr: {'Rgb': (255, 128, 0)}).
        assert_eq!(result.unwrap(), val);
    });
}

#[test]
fn test_enum_struct_variant_round_trip() {
    Python::attach(|py| {
        let val = Color::Named {
            name: String::from("crimson"),
        };
        let py_obj = pythonize(py, &val).expect("pythonize Color::Named failed");
        let repr = py_obj.repr().unwrap().to_string();
        eprintln!("FR-013 struct variant repr: {repr}");
        let result: Result<Color, _> = depythonize(&py_obj);
        // BASELINE: Color::Named serialises as {'Named': {'name': 'crimson'}}
        // (observed repr: {'Named': {'name': 'crimson'}}).
        assert_eq!(result.unwrap(), val);
    });
}
