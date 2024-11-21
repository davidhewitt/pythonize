use std::marker::PhantomData;

use pyo3::types::{
    PyDict, PyDictMethods, PyList, PyListMethods, PyMapping, PySequence, PyString, PyTuple,
    PyTupleMethods,
};
use pyo3::{Bound, BoundObject, IntoPyObject, PyAny, PyResult, Python};
use serde::{ser, Serialize};

use crate::error::{PythonizeError, Result};

// TODO: move 'py lifetime into builder once GATs are available in MSRV
/// Trait for types which can represent a Python mapping
pub trait PythonizeMappingType<'py> {
    /// Builder type for Python mappings
    type Builder;

    /// Create a builder for a Python mapping
    fn builder(py: Python<'py>, len: Option<usize>) -> PyResult<Self::Builder>;

    /// Adds the key-value item to the mapping being built
    fn push_item(
        builder: &mut Self::Builder,
        key: Bound<'py, PyAny>,
        value: Bound<'py, PyAny>,
    ) -> PyResult<()>;

    /// Build the Python mapping
    fn finish(builder: Self::Builder) -> PyResult<Bound<'py, PyMapping>>;
}

// TODO: move 'py lifetime into builder once GATs are available in MSRV
/// Trait for types which can represent a Python mapping and have a name
pub trait PythonizeNamedMappingType<'py> {
    /// Builder type for Python mappings with a name
    type Builder;

    /// Create a builder for a Python mapping with a name
    fn builder(py: Python<'py>, len: usize, name: &'static str) -> PyResult<Self::Builder>;

    /// Adds the field to the named mapping being built
    fn push_field(
        builder: &mut Self::Builder,
        name: Bound<'py, PyString>,
        value: Bound<'py, PyAny>,
    ) -> PyResult<()>;

    /// Build the Python mapping
    fn finish(builder: Self::Builder) -> PyResult<Bound<'py, PyMapping>>;
}

/// Trait for types which can represent a Python sequence
pub trait PythonizeListType: Sized {
    /// Constructor
    fn create_sequence<'py, T, U>(
        py: Python<'py>,
        elements: impl IntoIterator<Item = T, IntoIter = U>,
    ) -> PyResult<Bound<PySequence>>
    where
        T: IntoPyObject<'py>,
        U: ExactSizeIterator<Item = T>;
}

// TODO: remove 'py lifetime once GATs are available in MSRV
/// Custom types for serialization
pub trait PythonizeTypes<'py> {
    /// Python map type (should be representable as python mapping)
    type Map: PythonizeMappingType<'py>;
    /// Python (struct-like) named map type (should be representable as python mapping)
    type NamedMap: PythonizeNamedMappingType<'py>;
    /// Python sequence type (should be representable as python sequence)
    type List: PythonizeListType;
}

impl<'py> PythonizeMappingType<'py> for PyDict {
    type Builder = Bound<'py, Self>;

    fn builder(py: Python<'py>, _len: Option<usize>) -> PyResult<Self::Builder> {
        Ok(Self::new(py))
    }

    fn push_item(
        builder: &mut Self::Builder,
        key: Bound<'py, PyAny>,
        value: Bound<'py, PyAny>,
    ) -> PyResult<()> {
        builder.set_item(key, value)
    }

    fn finish(builder: Self::Builder) -> PyResult<Bound<'py, PyMapping>> {
        Ok(builder.into_mapping())
    }
}

/// Adapter type to use an unnamed mapping type, i.e. one that implements
/// [`PythonizeMappingType`], as a named mapping type, i.e. one that implements
/// [`PythonizeNamedMappingType`]. The adapter simply drops the provided name.
///
/// This adapter is commonly applied to use the same unnamed mapping type for
/// both [`PythonizeTypes::Map`] and [`PythonizeTypes::NamedMap`] while only
/// implementing [`PythonizeMappingType`].
pub struct PythonizeUnnamedMappingAdapter<'py, T: PythonizeMappingType<'py>> {
    _unnamed: T,
    _marker: PhantomData<&'py ()>,
}

impl<'py, T: PythonizeMappingType<'py>> PythonizeNamedMappingType<'py>
    for PythonizeUnnamedMappingAdapter<'py, T>
{
    type Builder = <T as PythonizeMappingType<'py>>::Builder;

    fn builder(py: Python<'py>, len: usize, _name: &'static str) -> PyResult<Self::Builder> {
        <T as PythonizeMappingType>::builder(py, Some(len))
    }

    fn push_field(
        builder: &mut Self::Builder,
        name: Bound<'py, PyString>,
        value: Bound<'py, PyAny>,
    ) -> PyResult<()> {
        <T as PythonizeMappingType>::push_item(builder, name.into_any(), value)
    }

    fn finish(builder: Self::Builder) -> PyResult<Bound<'py, PyMapping>> {
        <T as PythonizeMappingType>::finish(builder)
    }
}

impl PythonizeListType for PyList {
    fn create_sequence<'py, T, U>(
        py: Python<'py>,
        elements: impl IntoIterator<Item = T, IntoIter = U>,
    ) -> PyResult<Bound<PySequence>>
    where
        T: IntoPyObject<'py>,
        U: ExactSizeIterator<Item = T>,
    {
        Ok(PyList::new(py, elements)?.into_sequence())
    }
}

impl PythonizeListType for PyTuple {
    fn create_sequence<'py, T, U>(
        py: Python<'py>,
        elements: impl IntoIterator<Item = T, IntoIter = U>,
    ) -> PyResult<Bound<PySequence>>
    where
        T: IntoPyObject<'py>,
        U: ExactSizeIterator<Item = T>,
    {
        Ok(PyTuple::new(py, elements)?.into_sequence())
    }
}

pub struct PythonizeDefault;

impl<'py> PythonizeTypes<'py> for PythonizeDefault {
    type Map = PyDict;
    type NamedMap = PythonizeUnnamedMappingAdapter<'py, PyDict>;
    type List = PyList;
}

/// Attempt to convert the given data into a Python object
pub fn pythonize<'py, T>(py: Python<'py>, value: &T) -> Result<Bound<'py, PyAny>>
where
    T: ?Sized + Serialize,
{
    value.serialize(Pythonizer::new(py))
}

/// Attempt to convert the given data into a Python object.
/// Also uses custom mapping python class for serialization.
pub fn pythonize_custom<'py, P, T>(py: Python<'py>, value: &T) -> Result<Bound<'py, PyAny>>
where
    T: ?Sized + Serialize,
    P: PythonizeTypes<'py>,
{
    value.serialize(Pythonizer::custom::<P>(py))
}

/// A structure that serializes Rust values into Python objects
#[derive(Clone, Copy)]
pub struct Pythonizer<'py, P> {
    py: Python<'py>,
    _types: PhantomData<P>,
}

impl<'py, P> From<Python<'py>> for Pythonizer<'py, P> {
    fn from(py: Python<'py>) -> Self {
        Self {
            py,
            _types: PhantomData,
        }
    }
}

impl<'py> Pythonizer<'py, PythonizeDefault> {
    /// Creates a serializer to convert data into a Python object using the default mapping class
    pub fn new(py: Python<'py>) -> Self {
        Self::from(py)
    }

    /// Creates a serializer to convert data into a Python object using a custom mapping class
    pub fn custom<P>(py: Python<'py>) -> Pythonizer<'py, P> {
        Pythonizer::from(py)
    }
}

#[doc(hidden)]
pub struct PythonCollectionSerializer<'py, P> {
    items: Vec<Bound<'py, PyAny>>,
    py: Python<'py>,
    _types: PhantomData<P>,
}

#[doc(hidden)]
pub struct PythonTupleVariantSerializer<'py, P> {
    name: &'static str,
    variant: &'static str,
    inner: PythonCollectionSerializer<'py, P>,
}

#[doc(hidden)]
pub struct PythonStructVariantSerializer<'py, P: PythonizeTypes<'py>> {
    name: &'static str,
    variant: &'static str,
    inner: PythonStructDictSerializer<'py, P>,
}

#[doc(hidden)]
pub struct PythonStructDictSerializer<'py, P: PythonizeTypes<'py>> {
    py: Python<'py>,
    builder: <P::NamedMap as PythonizeNamedMappingType<'py>>::Builder,
    _types: PhantomData<P>,
}

#[doc(hidden)]
pub struct PythonMapSerializer<'py, P: PythonizeTypes<'py>> {
    py: Python<'py>,
    builder: <P::Map as PythonizeMappingType<'py>>::Builder,
    key: Option<Bound<'py, PyAny>>,
    _types: PhantomData<P>,
}

impl<'py, P: PythonizeTypes<'py>> Pythonizer<'py, P> {
    /// The default implementation for serialisation functions.
    #[inline]
    fn serialise_default<T>(self, v: T) -> Result<Bound<'py, PyAny>>
    where
        T: IntoPyObject<'py>,
        <T as IntoPyObject<'py>>::Error: Into<PythonizeError>,
    {
        v.into_pyobject(self.py)
            .map(|x| x.into_any().into_bound())
            .map_err(Into::into)
    }
}

impl<'py, P: PythonizeTypes<'py>> ser::Serializer for Pythonizer<'py, P> {
    type Ok = Bound<'py, PyAny>;
    type Error = PythonizeError;
    type SerializeSeq = PythonCollectionSerializer<'py, P>;
    type SerializeTuple = PythonCollectionSerializer<'py, P>;
    type SerializeTupleStruct = PythonCollectionSerializer<'py, P>;
    type SerializeTupleVariant = PythonTupleVariantSerializer<'py, P>;
    type SerializeMap = PythonMapSerializer<'py, P>;
    type SerializeStruct = PythonStructDictSerializer<'py, P>;
    type SerializeStructVariant = PythonStructVariantSerializer<'py, P>;

    fn serialize_bool(self, v: bool) -> Result<Bound<'py, PyAny>> {
        self.serialise_default(v)
    }

    fn serialize_i8(self, v: i8) -> Result<Bound<'py, PyAny>> {
        self.serialise_default(v)
    }

    fn serialize_i16(self, v: i16) -> Result<Bound<'py, PyAny>> {
        self.serialise_default(v)
    }

    fn serialize_i32(self, v: i32) -> Result<Bound<'py, PyAny>> {
        self.serialise_default(v)
    }

    fn serialize_i64(self, v: i64) -> Result<Bound<'py, PyAny>> {
        self.serialise_default(v)
    }

    fn serialize_u8(self, v: u8) -> Result<Bound<'py, PyAny>> {
        self.serialise_default(v)
    }

    fn serialize_u16(self, v: u16) -> Result<Bound<'py, PyAny>> {
        self.serialise_default(v)
    }

    fn serialize_u32(self, v: u32) -> Result<Bound<'py, PyAny>> {
        self.serialise_default(v)
    }

    fn serialize_u64(self, v: u64) -> Result<Bound<'py, PyAny>> {
        self.serialise_default(v)
    }

    fn serialize_f32(self, v: f32) -> Result<Bound<'py, PyAny>> {
        self.serialise_default(v)
    }

    fn serialize_f64(self, v: f64) -> Result<Bound<'py, PyAny>> {
        self.serialise_default(v)
    }

    fn serialize_char(self, v: char) -> Result<Bound<'py, PyAny>> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Bound<'py, PyAny>> {
        Ok(PyString::new(self.py, v).into_any())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Bound<'py, PyAny>> {
        self.serialise_default(v)
    }

    fn serialize_none(self) -> Result<Bound<'py, PyAny>> {
        Ok(self.py.None().into_bound(self.py))
    }

    fn serialize_some<T>(self, value: &T) -> Result<Bound<'py, PyAny>>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Bound<'py, PyAny>> {
        self.serialize_none()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Bound<'py, PyAny>> {
        self.serialize_none()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Bound<'py, PyAny>> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Bound<'py, PyAny>>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Bound<'py, PyAny>>
    where
        T: ?Sized + Serialize,
    {
        let mut m = P::NamedMap::builder(self.py, 1, name)?;
        P::NamedMap::push_field(
            &mut m,
            PyString::new(self.py, variant),
            value.serialize(self)?,
        )?;
        Ok(P::NamedMap::finish(m)?.into_any())
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
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<PythonTupleVariantSerializer<'py, P>> {
        let inner = self.serialize_tuple(len)?;
        Ok(PythonTupleVariantSerializer {
            name,
            variant,
            inner,
        })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<PythonMapSerializer<'py, P>> {
        Ok(PythonMapSerializer {
            builder: P::Map::builder(self.py, len)?,
            key: None,
            py: self.py,
            _types: PhantomData,
        })
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<PythonStructDictSerializer<'py, P>> {
        Ok(PythonStructDictSerializer {
            py: self.py,
            builder: P::NamedMap::builder(self.py, len, name)?,
            _types: PhantomData,
        })
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<PythonStructVariantSerializer<'py, P>> {
        Ok(PythonStructVariantSerializer {
            name,
            variant,
            inner: PythonStructDictSerializer {
                py: self.py,
                builder: P::NamedMap::builder(self.py, len, variant)?,
                _types: PhantomData,
            },
        })
    }
}

impl<'py, P: PythonizeTypes<'py>> ser::SerializeSeq for PythonCollectionSerializer<'py, P> {
    type Ok = Bound<'py, PyAny>;
    type Error = PythonizeError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.items.push(pythonize_custom::<P, _>(self.py, value)?);
        Ok(())
    }

    fn end(self) -> Result<Bound<'py, PyAny>> {
        let instance = P::List::create_sequence(self.py, self.items)?;
        Ok(instance.into_pyobject(self.py)?.into_any())
    }
}

impl<'py, P: PythonizeTypes<'py>> ser::SerializeTuple for PythonCollectionSerializer<'py, P> {
    type Ok = Bound<'py, PyAny>;
    type Error = PythonizeError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Bound<'py, PyAny>> {
        Ok(PyTuple::new(self.py, self.items)?.into_any())
    }
}

impl<'py, P: PythonizeTypes<'py>> ser::SerializeTupleStruct for PythonCollectionSerializer<'py, P> {
    type Ok = Bound<'py, PyAny>;
    type Error = PythonizeError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Bound<'py, PyAny>> {
        ser::SerializeTuple::end(self)
    }
}

impl<'py, P: PythonizeTypes<'py>> ser::SerializeTupleVariant
    for PythonTupleVariantSerializer<'py, P>
{
    type Ok = Bound<'py, PyAny>;
    type Error = PythonizeError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(&mut self.inner, value)
    }

    fn end(self) -> Result<Bound<'py, PyAny>> {
        let mut m = P::NamedMap::builder(self.inner.py, 1, self.name)?;
        P::NamedMap::push_field(
            &mut m,
            PyString::new(self.inner.py, self.variant),
            ser::SerializeTuple::end(self.inner)?,
        )?;
        Ok(P::NamedMap::finish(m)?.into_any())
    }
}

impl<'py, P: PythonizeTypes<'py>> ser::SerializeMap for PythonMapSerializer<'py, P> {
    type Ok = Bound<'py, PyAny>;
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
        P::Map::push_item(
            &mut self.builder,
            self.key
                .take()
                .expect("serialize_value should always be called after serialize_key"),
            pythonize_custom::<P, _>(self.py, value)?,
        )?;
        Ok(())
    }

    fn end(self) -> Result<Bound<'py, PyAny>> {
        Ok(P::Map::finish(self.builder)?.into_any())
    }
}

impl<'py, P: PythonizeTypes<'py>> ser::SerializeStruct for PythonStructDictSerializer<'py, P> {
    type Ok = Bound<'py, PyAny>;
    type Error = PythonizeError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        P::NamedMap::push_field(
            &mut self.builder,
            PyString::new(self.py, key),
            pythonize_custom::<P, _>(self.py, value)?,
        )?;
        Ok(())
    }

    fn end(self) -> Result<Bound<'py, PyAny>> {
        Ok(P::NamedMap::finish(self.builder)?.into_any())
    }
}

impl<'py, P: PythonizeTypes<'py>> ser::SerializeStructVariant
    for PythonStructVariantSerializer<'py, P>
{
    type Ok = Bound<'py, PyAny>;
    type Error = PythonizeError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        P::NamedMap::push_field(
            &mut self.inner.builder,
            PyString::new(self.inner.py, key),
            pythonize_custom::<P, _>(self.inner.py, value)?,
        )?;
        Ok(())
    }

    fn end(self) -> Result<Bound<'py, PyAny>> {
        let v = P::NamedMap::finish(self.inner.builder)?;
        let mut m = P::NamedMap::builder(self.inner.py, 1, self.name)?;
        P::NamedMap::push_field(
            &mut m,
            PyString::new(self.inner.py, self.variant),
            v.into_any(),
        )?;
        Ok(P::NamedMap::finish(m)?.into_any())
    }
}

#[cfg(test)]
mod test {
    use super::pythonize;
    use maplit::hashmap;
    use pyo3::ffi::c_str;
    use pyo3::prelude::*;
    use pyo3::pybacked::PyBackedStr;
    use pyo3::types::{PyBytes, PyDict};
    use serde::Serialize;

    fn test_ser<T>(src: T, expected: &str)
    where
        T: Serialize,
    {
        Python::with_gil(|py| -> PyResult<()> {
            let obj = pythonize(py, &src)?;

            let locals = PyDict::new(py);
            locals.set_item("obj", obj)?;

            py.run(
                c_str!("import json; result = json.dumps(obj, separators=(',', ':'))"),
                None,
                Some(&locals),
            )?;
            let result = locals.get_item("result")?.unwrap();
            let result = result.extract::<PyBackedStr>()?;

            assert_eq!(result, expected);
            assert_eq!(serde_json::to_string(&src).unwrap(), expected);

            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_empty_struct() {
        #[derive(Serialize)]
        struct Empty;

        test_ser(Empty, "null");
    }

    #[test]
    fn test_struct() {
        #[derive(Serialize)]
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
    fn test_nested_struct() {
        #[derive(Serialize)]
        struct Foo {
            name: String,
            bar: Bar,
        }

        #[derive(Serialize)]
        struct Bar {
            name: String,
        }

        test_ser(
            Foo {
                name: "foo".to_string(),
                bar: Bar {
                    name: "bar".to_string(),
                },
            },
            r#"{"name":"foo","bar":{"name":"bar"}}"#,
        )
    }

    #[test]
    fn test_tuple_struct() {
        #[derive(Serialize)]
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
        #[derive(Serialize)]
        enum E {
            Empty,
        }

        test_ser(E::Empty, r#""Empty""#);
    }

    #[test]
    fn test_enum_tuple_variant() {
        #[derive(Serialize)]
        enum E {
            Tuple(i32, String),
        }

        test_ser(E::Tuple(5, "foo".to_string()), r#"{"Tuple":[5,"foo"]}"#);
    }

    #[test]
    fn test_enum_newtype_variant() {
        #[derive(Serialize)]
        enum E {
            NewType(String),
        }

        test_ser(E::NewType("foo".to_string()), r#"{"NewType":"foo"}"#);
    }

    #[test]
    fn test_enum_struct_variant() {
        #[derive(Serialize)]
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

    #[test]
    fn test_integers() {
        #[derive(Serialize)]
        struct Integers {
            a: i8,
            b: i16,
            c: i32,
            d: i64,
            e: u8,
            f: u16,
            g: u32,
            h: u64,
        }

        test_ser(
            Integers {
                a: 1,
                b: 2,
                c: 3,
                d: 4,
                e: 5,
                f: 6,
                g: 7,
                h: 8,
            },
            r#"{"a":1,"b":2,"c":3,"d":4,"e":5,"f":6,"g":7,"h":8}"#,
        )
    }

    #[test]
    fn test_floats() {
        #[derive(Serialize)]
        struct Floats {
            a: f32,
            b: f64,
        }

        test_ser(Floats { a: 1.0, b: 2.0 }, r#"{"a":1.0,"b":2.0}"#)
    }

    #[test]
    fn test_char() {
        #[derive(Serialize)]
        struct Char {
            a: char,
        }

        test_ser(Char { a: 'a' }, r#"{"a":"a"}"#)
    }

    #[test]
    fn test_bool() {
        test_ser(true, "true");
        test_ser(false, "false");
    }

    #[test]
    fn test_none() {
        #[derive(Serialize)]
        struct S;

        test_ser((), "null");
        test_ser(S, "null");

        test_ser(Some(1), "1");
        test_ser(None::<i32>, "null");
    }

    #[test]
    fn test_bytes() {
        // serde treats &[u8] as a sequence of integers due to lack of specialization
        test_ser(b"foo", "[102,111,111]");

        Python::with_gil(|py| {
            assert!(pythonize(py, serde_bytes::Bytes::new(b"foo"))
                .expect("bytes will always serialize successfully")
                .eq(&PyBytes::new(py, b"foo"))
                .expect("bytes will always compare successfully"));
        });
    }
}
