use pyo3::prelude::*;
use pythonize::{depythonize, pythonize};

// ---------------------------------------------------------------------------
// Case 1 — Non-Latin script round-trips
// Normative: String values containing non-Latin scripts must round-trip
// byte-exactly.  These are not characterisation tests.
// ---------------------------------------------------------------------------

#[test]
fn test_string_cyrillic_round_trip() {
    Python::attach(|py| {
        let val = String::from("Привет"); // U+041F U+0440 U+0438 U+0432 U+0435 U+0442
        let result = pythonize(py, &val).and_then(|py_val| depythonize::<String>(&py_val));
        assert_eq!(result.unwrap(), val);
    });
}

#[test]
fn test_string_cjk_round_trip() {
    Python::attach(|py| {
        let val = String::from("你好"); // U+4F60 U+597D
        let result = pythonize(py, &val).and_then(|py_val| depythonize::<String>(&py_val));
        assert_eq!(result.unwrap(), val);
    });
}

#[test]
fn test_string_arabic_round_trip() {
    Python::attach(|py| {
        let val = String::from("مرحبا"); // U+0645 U+0631 U+062D U+0628 U+0627
        let result = pythonize(py, &val).and_then(|py_val| depythonize::<String>(&py_val));
        assert_eq!(result.unwrap(), val);
    });
}

#[test]
fn test_string_devanagari_round_trip() {
    Python::attach(|py| {
        let val = String::from("नमस्ते"); // U+0928 U+092E U+0938 U+094D U+0924 U+0947
        let result = pythonize(py, &val).and_then(|py_val| depythonize::<String>(&py_val));
        assert_eq!(result.unwrap(), val);
    });
}

#[test]
fn test_string_hebrew_round_trip() {
    Python::attach(|py| {
        let val = String::from("שלום"); // U+05E9 U+05DC U+05D5 U+05DD
        let result = pythonize(py, &val).and_then(|py_val| depythonize::<String>(&py_val));
        assert_eq!(result.unwrap(), val);
    });
}

// ---------------------------------------------------------------------------
// Case 2 — Homoglyphs are preserved as their original codepoint
// BASELINE: Python does not normalise homoglyphs; each codepoint round-trips
// to itself with no cross-codepoint equality.
// ---------------------------------------------------------------------------

#[test]
fn test_string_latin_capital_a_round_trip() {
    Python::attach(|py| {
        let val = String::from("\u{0041}"); // Latin capital A (U+0041)
        let result = pythonize(py, &val).and_then(|py_val| depythonize::<String>(&py_val));
        // BASELINE: round-trip preserves U+0041; Python does not normalise to another A-homoglyph.
        assert_eq!(result.unwrap(), val);
    });
}

#[test]
fn test_string_cyrillic_capital_a_round_trip() {
    Python::attach(|py| {
        // Cyrillic capital А (U+0410) — visually identical to Latin A (U+0041)
        let val = String::from("\u{0410}");
        let result = pythonize(py, &val).and_then(|py_val| depythonize::<String>(&py_val));
        // BASELINE: round-trip preserves U+0410; Python does not rewrite to U+0041.
        assert_eq!(result.unwrap(), val);
    });
}

#[test]
fn test_string_fullwidth_latin_capital_a_round_trip() {
    Python::attach(|py| {
        let val = String::from("\u{FF21}"); // Fullwidth Latin capital Ａ (U+FF21)
        let result = pythonize(py, &val).and_then(|py_val| depythonize::<String>(&py_val));
        // BASELINE: round-trip preserves U+FF21; Python does not rewrite to U+0041.
        assert_eq!(result.unwrap(), val);
    });
}

// ---------------------------------------------------------------------------
// Case 3 — NFD is preserved, no silent NFC recomposition
// Confirmed baseline: CPython's PyString::new / to_cow() are length-aware;
// no Unicode normalisation is applied.  NFD in → NFD out.
// ---------------------------------------------------------------------------

#[test]
fn test_string_nfd_combining_acute_preserved() {
    Python::attach(|py| {
        // NFD: 'e' (U+0065, 1 byte) + combining acute accent (U+0301, 2 bytes) = 3 bytes.
        // Do NOT use the precomposed "é" (U+00E9, NFC, 2 bytes) — that would be trivial.
        let val = String::from("e\u{0301}");
        let result = pythonize(py, &val).and_then(|py_val| depythonize::<String>(&py_val));
        // BASELINE (confirmed 2026-04-04): CPython preserves NFD; no NFC recomposition.
        assert!(result.is_ok());
        let back = result.unwrap();
        assert_eq!(back, "e\u{0301}");
        assert_eq!(back.len(), 3); // 3 bytes in UTF-8; NFC "é" (U+00E9) would be 2
    });
}

// ---------------------------------------------------------------------------
// Case 4 — Non-BMP emoji round-trip
// ---------------------------------------------------------------------------

#[test]
fn test_string_non_bmp_emoji_round_trip() {
    Python::attach(|py| {
        // Non-BMP: 4 UTF-8 bytes; Python 3 str handles this natively.
        let val = String::from("🦀"); // U+1F980, Rust str len = 4
        let result = pythonize(py, &val).and_then(|py_val| depythonize::<String>(&py_val));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "🦀");
    });
}

// ---------------------------------------------------------------------------
// Case 5 — Null byte survives round-trip
// Confirmed baseline: pythonize uses length-aware CPython APIs; embedded null
// is not a string terminator.
// ---------------------------------------------------------------------------

#[test]
fn test_string_embedded_null_round_trip() {
    Python::attach(|py| {
        let val = String::from("hello\x00world");
        let result = pythonize(py, &val).and_then(|py_val| depythonize::<String>(&py_val));
        // BASELINE (confirmed 2026-04-04): pythonize uses length-aware CPython APIs;
        // embedded null is not a string terminator.
        // Caveat: truncation could occur if an intermediate consumer uses
        // PyUnicode_AsUTF8 (no-size variant) — not pythonize's bug.
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello\x00world");
    });
}
