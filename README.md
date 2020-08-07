# Pythonize

***WIP: Most functionality of this crate is still stubbed out. Please don't try to use this yet - unless you're interested in submitting PRs to help finish it off :)***

This is an experimental serializer for Rust's serde ecosystem, which can convert Rust objects to Python values and back.

At the moment the Python structures it produces should be _very_ similar to those which are produced by `serde_json`; i.e. calling Python's `json.loads()` on a value encoded by `serde_json` should produce an identical structure to
that which is produced directly by `pythonize`.

## Usage

Pythonize has two public APIs: `pythonize` and `depythonize`.

```
// TODO Example
```
