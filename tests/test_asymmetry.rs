use std::collections::BTreeMap;

use pyo3::prelude::*;
use pyo3::types::PyString;
use pythonize::{depythonize, pythonize};

// ---------------------------------------------------------------------------
// AC-1: Dict ordering — BTreeMap<String, i32> round-trip
// ---------------------------------------------------------------------------

#[test]
fn test_btreemap_round_trip_content_equality() {
    Python::attach(|py| {
        let mut map = BTreeMap::new();
        map.insert("alpha".to_string(), 1i32);
        map.insert("beta".to_string(), 2i32);
        map.insert("gamma".to_string(), 3i32);

        let py_val = pythonize(py, &map).expect("pythonize failed");
        let result: Result<BTreeMap<String, i32>, _> = depythonize(&py_val);
        assert!(result.is_ok());
        let back = result.unwrap();
        // BASELINE: content equal — BTreeMap serializes keys in sorted order
        // (alpha, beta, gamma); Python dict (3.7+) preserves insertion order;
        // round-trip into BTreeMap re-sorts on insert.  Content matches.
        assert_eq!(back, map);
    });
}

// ---------------------------------------------------------------------------
// AC-2: f32 precision — round-trip characterisation (is_ok only)
// ---------------------------------------------------------------------------

#[test]
fn test_f32_min_positive_round_trip_is_ok() {
    Python::attach(|py| {
        let val = f32::MIN_POSITIVE;
        let py_val = pythonize(py, &val).expect("pythonize failed");
        let result: Result<f32, _> = depythonize(&py_val);
        // BASELINE: Ok(1.1754944e-38) — f32 → f64 (exact promotion) → Python
        // float → f64 → f32; normal value round-trips without error.
        assert!(result.is_ok());
    });
}

#[test]
fn test_f32_pi_round_trip_is_ok() {
    Python::attach(|py| {
        let val = std::f32::consts::PI;
        let py_val = pythonize(py, &val).expect("pythonize failed");
        let result: Result<f32, _> = depythonize(&py_val);
        // BASELINE: Ok(3.1415927) — PI in f32 precision; f32→f64 promotion is
        // lossless so back-conversion yields the original f32 bit pattern.
        assert!(result.is_ok());
    });
}

#[test]
fn test_f32_subnormal_round_trip_is_ok() {
    Python::attach(|py| {
        // Smallest positive subnormal f32 (~1.4e-45)
        let val = f32::from_bits(1u32);
        let py_val = pythonize(py, &val).expect("pythonize failed");
        let result: Result<f32, _> = depythonize(&py_val);
        // BASELINE: Ok(1e-45) — subnormal f32 is representable in f64;
        // round-trip completes without error.
        assert!(result.is_ok());
    });
}

// ---------------------------------------------------------------------------
// AC-3: char length guard — Python str with >1 char → depythonize::<char>
// ---------------------------------------------------------------------------

#[test]
fn test_depythonize_two_char_ascii_as_char_is_err() {
    Python::attach(|py| {
        let py_str = PyString::new(py, "ab");
        let result: Result<char, _> = depythonize(py_str.as_any());
        // assert!(err.to_string().contains("expected a str of length 1 for char"))
        assert!(result.is_err());
    });
}

#[test]
fn test_depythonize_multi_char_unicode_as_char_is_err() {
    Python::attach(|py| {
        let py_str = PyString::new(py, "hello");
        let result: Result<char, _> = depythonize(py_str.as_any());
        // assert!(err.to_string().contains("expected a str of length 1 for char"))
        assert!(result.is_err());
    });
}

// ---------------------------------------------------------------------------
// AC-4: bool as int — Python bool → depythonize::<i64>  (normative)
// ---------------------------------------------------------------------------

#[test]
fn test_depythonize_py_true_as_i64_is_one() {
    Python::attach(|py| {
        let py_true = pythonize(py, &true).unwrap();
        let result: Result<i64, _> = depythonize(&py_true);
        // BASELINE: Ok(1) — Python True is a subclass of int; the
        // deserializer reads it as integer 1.
        assert_eq!(result.unwrap(), 1i64);
    });
}

#[test]
fn test_depythonize_py_false_as_i64_is_zero() {
    Python::attach(|py| {
        let py_false = pythonize(py, &false).unwrap();
        let result: Result<i64, _> = depythonize(&py_false);
        // BASELINE: Ok(0) — Python False is a subclass of int; the
        // deserializer reads it as integer 0.
        assert_eq!(result.unwrap(), 0i64);
    });
}

// ---------------------------------------------------------------------------
// Additional: Rust bool → Python → back as bool (round-trip)
// ---------------------------------------------------------------------------

#[test]
fn test_bool_round_trip_true() {
    Python::attach(|py| {
        let py_val = pythonize(py, &true).unwrap();
        let result: Result<bool, _> = depythonize(&py_val);
        // BASELINE: Ok(true) — bool round-trip works correctly.
        assert_eq!(result.unwrap(), true);
    });
}

#[test]
fn test_bool_round_trip_false() {
    Python::attach(|py| {
        let py_val = pythonize(py, &false).unwrap();
        let result: Result<bool, _> = depythonize(&py_val);
        // BASELINE: Ok(false) — bool round-trip works correctly.
        assert_eq!(result.unwrap(), false);
    });
}

// ---------------------------------------------------------------------------
// Additional: Rust bool → Python → depythonize::<i32>
// ---------------------------------------------------------------------------

#[test]
fn test_depythonize_py_true_as_i32() {
    Python::attach(|py| {
        let py_true = pythonize(py, &true).unwrap();
        let result: Result<i32, _> = depythonize(&py_true);
        // BASELINE: Ok(1) — Python True subclasses int; coerces to i32.
        assert_eq!(result.unwrap(), 1i32);
    });
}

#[test]
fn test_depythonize_py_false_as_i32() {
    Python::attach(|py| {
        let py_false = pythonize(py, &false).unwrap();
        let result: Result<i32, _> = depythonize(&py_false);
        // BASELINE: Ok(0) — Python False subclasses int; coerces to i32.
        assert_eq!(result.unwrap(), 0i32);
    });
}

// ---------------------------------------------------------------------------
// Additional: Python int 300 → depythonize::<i8> / ::<u8> (overflow probe)
// ---------------------------------------------------------------------------

#[test]
fn test_depythonize_int_300_as_i8_overflow() {
    Python::attach(|py| {
        let py_int = pythonize(py, &300i32).unwrap();
        let result: Result<i8, _> = depythonize(&py_int);
        // BASELINE: Err — 300 > i8::MAX (127); overflow returns an error.
        assert!(result.is_err());
    });
}

#[test]
fn test_depythonize_int_300_as_u8_overflow() {
    Python::attach(|py| {
        let py_int = pythonize(py, &300i32).unwrap();
        let result: Result<u8, _> = depythonize(&py_int);
        // BASELINE: Err — 300 > u8::MAX (255); overflow returns an error.
        assert!(result.is_err());
    });
}

// ---------------------------------------------------------------------------
// Additional: f64::NAN → pythonize → depythonize::<f32>
// ---------------------------------------------------------------------------

#[test]
fn test_f64_nan_to_f32_via_python() {
    Python::attach(|py| {
        let nan_f64 = f64::NAN;
        let py_val = pythonize(py, &nan_f64).expect("pythonize failed");
        let result: Result<f32, _> = depythonize(&py_val);
        // BASELINE: Ok(f32::NAN) — f64 NaN → Python float nan → f32 NaN.
        // NaN != NaN so equality is verified with is_nan().
        assert!(result.is_ok());
        assert!(result.unwrap().is_nan());
    });
}

// ---------------------------------------------------------------------------
// Additional: Rust () (unit) → Python → back as ()
// ---------------------------------------------------------------------------

#[test]
fn test_unit_round_trip() {
    Python::attach(|py| {
        let py_val = pythonize(py, &()).expect("pythonize failed");
        let result: Result<(), _> = depythonize(&py_val);
        // BASELINE: Ok(()) — unit serializes to Python None; None correctly
        // deserializes back to ().  Round-trip succeeds.
        assert!(result.is_ok());
    });
}
