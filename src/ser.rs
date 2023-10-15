use std::marker::PhantomData;

use pyo3::types::{PyDict, PyList, PyMapping, PySequence, PyTuple};
use pyo3::{IntoPy, PyObject, PyResult, Python, ToPyObject};
use serde::{ser, Serialize};

use crate::error::{PythonizeError, Result};

/// Trait for types which can represent a Python mapping
pub trait PythonizeDictType {
    /// Constructor
    fn create_mapping(py: Python) -> PyResult<&PyMapping>;
}

/// Trait for types which can represent a Python sequence
pub trait PythonizeListType: Sized {
    /// Constructor
    fn create_sequence<T, U>(
        py: Python,
        elements: impl IntoIterator<Item = T, IntoIter = U>,
    ) -> PyResult<&PySequence>
    where
        T: ToPyObject,
        U: ExactSizeIterator<Item = T>;
}

/// Custom types for serialization
pub trait PythonizeTypes {
    /// Python map type (should be representable as python mapping)
    type Map: PythonizeDictType;
    /// Python sequence type (should be representable as python sequence)
    type List: PythonizeListType;
}

impl PythonizeDictType for PyDict {
    fn create_mapping(py: Python) -> PyResult<&PyMapping> {
        Ok(PyDict::new(py).as_mapping())
    }
}

impl PythonizeListType for PyList {
    fn create_sequence<T, U>(
        py: Python,
        elements: impl IntoIterator<Item = T, IntoIter = U>,
    ) -> PyResult<&PySequence>
    where
        T: ToPyObject,
        U: ExactSizeIterator<Item = T>,
    {
        Ok(PyList::new(py, elements).as_sequence())
    }
}

struct PythonizeDefault;

impl PythonizeTypes for PythonizeDefault {
    type Map = PyDict;
    type List = PyList;
}

/// Attempt to convert the given data into a Python object
pub fn pythonize<T>(py: Python, value: &T) -> Result<PyObject>
where
    T: ?Sized + Serialize,
{
    pythonize_custom::<PythonizeDefault, _>(py, value)
}

/// Attempt to convert the given data into a Python object.
/// Also uses custom mapping python class for serialization.
pub fn pythonize_custom<P, T>(py: Python, value: &T) -> Result<PyObject>
where
    T: ?Sized + Serialize,
    P: PythonizeTypes,
{
    value.serialize(Pythonizer::<P> {
        py,
        _types: PhantomData,
    })
}

#[derive(Clone, Copy)]
pub struct Pythonizer<'py, P> {
    py: Python<'py>,
    _types: PhantomData<P>,
}

#[doc(hidden)]
pub struct PythonCollectionSerializer<'py, P> {
    items: Vec<PyObject>,
    py: Python<'py>,
    _types: PhantomData<P>,
}

#[doc(hidden)]
pub struct PythonTupleVariantSerializer<'py, P> {
    variant: &'static str,
    inner: PythonCollectionSerializer<'py, P>,
}

#[doc(hidden)]
pub struct PythonStructVariantSerializer<'py, P: PythonizeTypes> {
    variant: &'static str,
    inner: PythonDictSerializer<'py, P>,
}

#[doc(hidden)]
pub struct PythonDictSerializer<'py, P: PythonizeTypes> {
    py: Python<'py>,
    dict: &'py PyMapping,
    _types: PhantomData<P>,
}

#[doc(hidden)]
pub struct PythonMapSerializer<'py, P: PythonizeTypes> {
    py: Python<'py>,
    map: &'py PyMapping,
    key: Option<PyObject>,
    _types: PhantomData<P>,
}

impl<'py, P: PythonizeTypes> ser::Serializer for Pythonizer<'py, P> {
    type Ok = PyObject;
    type Error = PythonizeError;
    type SerializeSeq = PythonCollectionSerializer<'py, P>;
    type SerializeTuple = PythonCollectionSerializer<'py, P>;
    type SerializeTupleStruct = PythonCollectionSerializer<'py, P>;
    type SerializeTupleVariant = PythonTupleVariantSerializer<'py, P>;
    type SerializeMap = PythonMapSerializer<'py, P>;
    type SerializeStruct = PythonDictSerializer<'py, P>;
    type SerializeStructVariant = PythonStructVariantSerializer<'py, P>;

    fn serialize_bool(self, v: bool) -> Result<PyObject> {
        Ok(v.into_py(self.py))
    }

    fn serialize_i8(self, v: i8) -> Result<PyObject> {
        Ok(v.into_py(self.py))
    }

    fn serialize_i16(self, v: i16) -> Result<PyObject> {
        Ok(v.into_py(self.py))
    }

    fn serialize_i32(self, v: i32) -> Result<PyObject> {
        Ok(v.into_py(self.py))
    }

    fn serialize_i64(self, v: i64) -> Result<PyObject> {
        Ok(v.into_py(self.py))
    }

    fn serialize_u8(self, v: u8) -> Result<PyObject> {
        Ok(v.into_py(self.py))
    }

    fn serialize_u16(self, v: u16) -> Result<PyObject> {
        Ok(v.into_py(self.py))
    }

    fn serialize_u32(self, v: u32) -> Result<PyObject> {
        Ok(v.into_py(self.py))
    }

    fn serialize_u64(self, v: u64) -> Result<PyObject> {
        Ok(v.into_py(self.py))
    }

    fn serialize_f32(self, v: f32) -> Result<PyObject> {
        Ok(v.into_py(self.py))
    }

    fn serialize_f64(self, v: f64) -> Result<PyObject> {
        Ok(v.into_py(self.py))
    }

    fn serialize_char(self, v: char) -> Result<PyObject> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<PyObject> {
        Ok(v.into_py(self.py))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<PyObject> {
        Ok(v.into_py(self.py))
    }

    fn serialize_none(self) -> Result<PyObject> {
        Ok(self.py.None())
    }

    fn serialize_some<T>(self, value: &T) -> Result<PyObject>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<PyObject> {
        self.serialize_none()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<PyObject> {
        self.serialize_none()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<PyObject> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<PyObject>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<PyObject>
    where
        T: ?Sized + Serialize,
    {
        let d = PyDict::new(self.py);
        d.set_item(variant, value.serialize(self)?)?;
        Ok(d.into())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<PythonCollectionSerializer<'py, P>> {
        let items = match len {
            Some(len) => Vec::with_capacity(len),
            None => Vec::new(),
        };
        Ok(PythonCollectionSerializer {
            items,
            py: self.py,
            _types: PhantomData,
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<PythonCollectionSerializer<'py, P>> {
        Ok(PythonCollectionSerializer {
            items: Vec::with_capacity(len),
            py: self.py,
            _types: PhantomData,
        })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<PythonCollectionSerializer<'py, P>> {
        self.serialize_tuple(len)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<PythonTupleVariantSerializer<'py, P>> {
        let inner = self.serialize_tuple(len)?;
        Ok(PythonTupleVariantSerializer { variant, inner })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<PythonMapSerializer<'py, P>> {
        Ok(PythonMapSerializer {
            map: P::Map::create_mapping(self.py)?,
            key: None,
            py: self.py,
            _types: PhantomData,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<PythonDictSerializer<'py, P>> {
        Ok(PythonDictSerializer {
            dict: P::Map::create_mapping(self.py)?,
            py: self.py,
            _types: PhantomData,
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<PythonStructVariantSerializer<'py, P>> {
        Ok(PythonStructVariantSerializer {
            variant,
            inner: PythonDictSerializer {
                dict: P::Map::create_mapping(self.py)?,
                py: self.py,
                _types: PhantomData,
            },
        })
    }
}

impl<'py, P: PythonizeTypes> ser::SerializeSeq for PythonCollectionSerializer<'py, P> {
    type Ok = PyObject;
    type Error = PythonizeError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.items.push(pythonize_custom::<P, _>(self.py, value)?);
        Ok(())
    }

    fn end(self) -> Result<PyObject> {
        let instance = P::List::create_sequence(self.py, self.items)?;
        Ok(instance.to_object(self.py))
    }
}

impl<'py, P: PythonizeTypes> ser::SerializeTuple for PythonCollectionSerializer<'py, P> {
    type Ok = PyObject;
    type Error = PythonizeError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<PyObject> {
        Ok(PyTuple::new(self.py, self.items).into())
    }
}

impl<'py, P: PythonizeTypes> ser::SerializeTupleStruct for PythonCollectionSerializer<'py, P> {
    type Ok = PyObject;
    type Error = PythonizeError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<PyObject> {
        ser::SerializeTuple::end(self)
    }
}

impl<'py, P: PythonizeTypes> ser::SerializeTupleVariant for PythonTupleVariantSerializer<'py, P> {
    type Ok = PyObject;
    type Error = PythonizeError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(&mut self.inner, value)
    }

    fn end(self) -> Result<PyObject> {
        let d = PyDict::new(self.inner.py);
        d.set_item(self.variant, ser::SerializeTuple::end(self.inner)?)?;
        Ok(d.into())
    }
}

impl<'py, P: PythonizeTypes> ser::SerializeMap for PythonMapSerializer<'py, P> {
    type Ok = PyObject;
    type Error = PythonizeError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.key = Some(pythonize_custom::<P, _>(self.py, key)?);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.map.set_item(
            self.key
                .take()
                .expect("serialize_value should always be called after serialize_key"),
            pythonize_custom::<P, _>(self.py, value)?,
        )?;
        Ok(())
    }

    fn end(self) -> Result<PyObject> {
        Ok(self.map.into())
    }
}

impl<'py, P: PythonizeTypes> ser::SerializeStruct for PythonDictSerializer<'py, P> {
    type Ok = PyObject;
    type Error = PythonizeError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Ok(self
            .dict
            .set_item(key, pythonize_custom::<P, _>(self.py, value)?)?)
    }

    fn end(self) -> Result<PyObject> {
        Ok(self.dict.into())
    }
}

impl<'py, P: PythonizeTypes> ser::SerializeStructVariant for PythonStructVariantSerializer<'py, P> {
    type Ok = PyObject;
    type Error = PythonizeError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.inner
            .dict
            .set_item(key, pythonize_custom::<P, _>(self.inner.py, value)?)?;
        Ok(())
    }

    fn end(self) -> Result<PyObject> {
        let d = PyDict::new(self.inner.py);
        d.set_item(self.variant, self.inner.dict)?;
        Ok(d.into())
    }
}

#[cfg(test)]
mod test {
    use super::pythonize;
    use maplit::hashmap;
    use pyo3::types::PyDict;
    use pyo3::{PyResult, Python};
    use serde::{Deserialize, Serialize};

    fn test_ser<T>(src: T, expected: &str)
    where
        T: Serialize,
    {
        Python::with_gil(|py| -> PyResult<()> {
            let obj = pythonize(py, &src)?;

            let locals = PyDict::new(py);
            locals.set_item("obj", obj)?;

            py.run(
                "import json; result = json.dumps(obj, separators=(',', ':'))",
                None,
                Some(locals),
            )?;
            let result = locals.get_item("result")?.unwrap().extract::<&str>()?;

            assert_eq!(result, expected);
            assert_eq!(serde_json::to_string(&src).unwrap(), expected);

            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_empty_struct() {
        #[derive(Serialize, Deserialize)]
        struct Empty;

        test_ser(Empty, "null");
    }

    #[test]
    fn test_struct() {
        #[derive(Serialize, Deserialize)]
        struct Struct {
            foo: String,
            bar: usize,
        }

        test_ser(
            Struct {
                foo: "foo".to_string(),
                bar: 5,
            },
            r#"{"foo":"foo","bar":5}"#,
        );
    }

    #[test]
    fn test_tuple_struct() {
        #[derive(Serialize, Deserialize)]
        struct TupleStruct(String, usize);

        test_ser(TupleStruct("foo".to_string(), 5), r#"["foo",5]"#);
    }

    #[test]
    fn test_tuple() {
        test_ser(("foo", 5), r#"["foo",5]"#);
    }

    #[test]
    fn test_vec() {
        test_ser(vec![1, 2, 3], r#"[1,2,3]"#);
    }

    #[test]
    fn test_map() {
        test_ser(hashmap! {"foo" => "foo"}, r#"{"foo":"foo"}"#);
    }

    #[test]
    fn test_enum_unit_variant() {
        #[derive(Serialize, Deserialize)]
        enum E {
            Empty,
        }

        test_ser(E::Empty, r#""Empty""#);
    }

    #[test]
    fn test_enum_tuple_variant() {
        #[derive(Serialize, Deserialize)]
        enum E {
            Tuple(i32, String),
        }

        test_ser(E::Tuple(5, "foo".to_string()), r#"{"Tuple":[5,"foo"]}"#);
    }

    #[test]
    fn test_enum_newtype_variant() {
        #[derive(Serialize, Deserialize)]
        enum E {
            NewType(String),
        }

        test_ser(E::NewType("foo".to_string()), r#"{"NewType":"foo"}"#);
    }

    #[test]
    fn test_enum_struct_variant() {
        #[derive(Serialize, Deserialize)]
        enum E {
            Struct { foo: String, bar: usize },
        }

        test_ser(
            E::Struct {
                foo: "foo".to_string(),
                bar: 5,
            },
            r#"{"Struct":{"foo":"foo","bar":5}}"#,
        );
    }
}
