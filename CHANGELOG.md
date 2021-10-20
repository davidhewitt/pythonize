## Unreleased

- Add `pythonize_custom` for customizing the Python types to serialize to.
- Add support for `depythonize` to handle arbitrary Python sequence and mapping types.

## 0.14.0 - 2021-07-05

- Update to PyO3 0.14

## 0.13.0 - 2020-12-28

- Update to PyO3 0.13

## 0.12.1 - 2020-12-08

- Require `std` feature of `serde`.
- Reduce memory consumption when deserializing sequences.
- Fix deserializing untagged struct enum variants.
- Fix deserializing sequences from Python tuples.

## 0.12.0 - 2020-11-22

- Change release versioning to match `pyo3` major/minor version.
- Implement `depythonizer`

## 0.1.0 - 2020-08-12

- Initial release
