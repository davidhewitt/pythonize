use pyo3::{types::*, Bound};
use serde::de::{self, DeserializeOwned, IntoDeserializer};
use serde::Deserialize;

use crate::error::{ErrorImpl, PythonizeError, Result};

/// Attempt to convert a Python object to an instance of `T`
pub fn depythonize<'a, 'py, T>(obj: &'a Bound<'py, PyAny>) -> Result<T>
where
    T: Deserialize<'a>,
{
    T::deserialize(&mut Depythonizer::from_object(obj))
}

/// Attempt to convert a Python object to an instance of `T`
#[deprecated(since = "0.22.0", note = "use `depythonize` instead")]
pub fn depythonize_bound<T>(obj: Bound<PyAny>) -> Result<T>
where
    T: DeserializeOwned,
{
    T::deserialize(&mut Depythonizer::from_object(&obj))
}

/// A structure that deserializes Python objects into Rust values
pub struct Depythonizer<'a, 'py> {
    input: &'a Bound<'py, PyAny>,
}

impl<'a, 'py> Depythonizer<'a, 'py> {
    /// Create a deserializer from a Python object
    pub fn from_object(input: &'a Bound<'py, PyAny>) -> Self {
        Depythonizer { input }
    }

    fn sequence_access(&self, expected_len: Option<usize>) -> Result<PySequenceAccess<'a, 'py>> {
        let seq = self.input.downcast::<PySequence>()?;
        let len = self.input.len()?;

        match expected_len {
            Some(expected) if expected != len => {
                Err(PythonizeError::incorrect_sequence_length(expected, len))
            }
            _ => Ok(PySequenceAccess::new(seq, len)),
        }
    }

    fn set_access(&self) -> Result<PySetAsSequence<'py>> {
        match self.input.downcast::<PySet>() {
            Ok(set) => Ok(PySetAsSequence::from_set(set)),
            Err(e) => {
                if let Ok(f) = self.input.downcast::<PyFrozenSet>() {
                    Ok(PySetAsSequence::from_frozenset(f))
                } else {
                    Err(e.into())
                }
            }
        }
    }

    fn dict_access(&self) -> Result<PyMappingAccess<'py>> {
        PyMappingAccess::new(self.input.downcast()?)
    }

    fn deserialize_any_int<'de, V>(&self, int: &Bound<'_, PyInt>, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if let Ok(x) = int.extract::<u128>() {
            if let Ok(x) = u8::try_from(x) {
                visitor.visit_u8(x)
            } else if let Ok(x) = u16::try_from(x) {
                visitor.visit_u16(x)
            } else if let Ok(x) = u32::try_from(x) {
                visitor.visit_u32(x)
            } else if let Ok(x) = u64::try_from(x) {
                visitor.visit_u64(x)
            } else {
                visitor.visit_u128(x)
            }
        } else {
            let x: i128 = int.extract()?;
            if let Ok(x) = i8::try_from(x) {
                visitor.visit_i8(x)
            } else if let Ok(x) = i16::try_from(x) {
                visitor.visit_i16(x)
            } else if let Ok(x) = i32::try_from(x) {
                visitor.visit_i32(x)
            } else if let Ok(x) = i64::try_from(x) {
                visitor.visit_i64(x)
            } else {
                visitor.visit_i128(x)
            }
        }
    }
}

macro_rules! deserialize_type {
    ($method:ident => $visit:ident) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value>
        where
            V: de::Visitor<'de>,
        {
            visitor.$visit(self.input.extract()?)
        }
    };
}

impl<'de> de::Deserializer<'de> for &'_ mut Depythonizer<'_, '_> {
    type Error = PythonizeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let obj = self.input;

        // First check for cases which are cheap to check due to pointer
        // comparison or bitflag checks
        if obj.is_none() {
            self.deserialize_unit(visitor)
        } else if obj.is_instance_of::<PyBool>() {
            self.deserialize_bool(visitor)
        } else if let Ok(x) = obj.downcast::<PyInt>() {
            self.deserialize_any_int(x, visitor)
        } else if obj.is_instance_of::<PyList>() || obj.is_instance_of::<PyTuple>() {
            self.deserialize_tuple(obj.len()?, visitor)
        } else if obj.is_instance_of::<PyDict>() {
            self.deserialize_map(visitor)
        } else if obj.is_instance_of::<PyString>() {
            self.deserialize_str(visitor)
        }
        // Continue with cases which are slower to check because they go
        // throuh `isinstance` machinery
        else if obj.is_instance_of::<PyBytes>() || obj.is_instance_of::<PyByteArray>() {
            self.deserialize_bytes(visitor)
        } else if obj.is_instance_of::<PyFloat>() {
            self.deserialize_f64(visitor)
        } else if obj.is_instance_of::<PyFrozenSet>() || obj.is_instance_of::<PySet>() {
            self.deserialize_seq(visitor)
        } else if obj.downcast::<PySequence>().is_ok() {
            self.deserialize_tuple(obj.len()?, visitor)
        } else if obj.downcast::<PyMapping>().is_ok() {
            self.deserialize_map(visitor)
        } else {
            Err(obj.get_type().qualname().map_or_else(
                |_| PythonizeError::unsupported_type("unknown"),
                PythonizeError::unsupported_type,
            ))
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_bool(self.input.is_truthy()?)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let s = self.input.downcast::<PyString>()?.to_cow()?;
        if s.len() != 1 {
            return Err(PythonizeError::invalid_length_char());
        }
        visitor.visit_char(s.chars().next().unwrap())
    }

    deserialize_type!(deserialize_i8 => visit_i8);
    deserialize_type!(deserialize_i16 => visit_i16);
    deserialize_type!(deserialize_i32 => visit_i32);
    deserialize_type!(deserialize_i64 => visit_i64);
    deserialize_type!(deserialize_i128 => visit_i128);
    deserialize_type!(deserialize_u8 => visit_u8);
    deserialize_type!(deserialize_u16 => visit_u16);
    deserialize_type!(deserialize_u32 => visit_u32);
    deserialize_type!(deserialize_u64 => visit_u64);
    deserialize_type!(deserialize_u128 => visit_u128);
    deserialize_type!(deserialize_f32 => visit_f32);
    deserialize_type!(deserialize_f64 => visit_f64);

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let s = self.input.downcast::<PyString>()?;
        visitor.visit_str(&s.to_cow()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let b = self.input.downcast::<PyBytes>()?;
        visitor.visit_bytes(b.as_bytes())
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.input.is_none() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.input.is_none() {
            visitor.visit_unit()
        } else {
            Err(PythonizeError::msg("expected None"))
        }
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.sequence_access(None) {
            Ok(seq) => visitor.visit_seq(seq),
            Err(e) => {
                // we allow sets to be deserialized as sequences, so try that
                if matches!(*e.inner, ErrorImpl::UnexpectedType(_)) {
                    if let Ok(set) = self.set_access() {
                        return visitor.visit_seq(set);
                    }
                }
                Err(e)
            }
        }
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(self.sequence_access(Some(len))?)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(self.sequence_access(Some(len))?)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(self.dict_access()?)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let item = &self.input;
        if let Ok(s) = item.downcast::<PyString>() {
            visitor.visit_enum(s.to_cow()?.into_deserializer())
        } else if let Ok(m) = item.downcast::<PyMapping>() {
            // Get the enum variant from the mapping key
            if m.len()? != 1 {
                return Err(PythonizeError::invalid_length_enum());
            }
            let variant: Bound<PyString> = m
                .keys()?
                .get_item(0)?
                .downcast_into::<PyString>()
                .map_err(|_| PythonizeError::dict_key_not_string())?;
            let value = m.get_item(&variant)?;
            visitor.visit_enum(PyEnumAccess::new(&value, variant))
        } else {
            Err(PythonizeError::invalid_enum_type())
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let s = self
            .input
            .downcast::<PyString>()
            .map_err(|_| PythonizeError::dict_key_not_string())?;
        visitor.visit_str(&s.to_cow()?)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_unit()
    }
}

struct PySequenceAccess<'a, 'py> {
    seq: &'a Bound<'py, PySequence>,
    index: usize,
    len: usize,
}

impl<'a, 'py> PySequenceAccess<'a, 'py> {
    fn new(seq: &'a Bound<'py, PySequence>, len: usize) -> Self {
        Self { seq, index: 0, len }
    }
}

impl<'de> de::SeqAccess<'de> for PySequenceAccess<'_, '_> {
    type Error = PythonizeError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.index < self.len {
            let item = self.seq.get_item(self.index)?;
            self.index += 1;
            seed.deserialize(&mut Depythonizer::from_object(&item))
                .map(Some)
        } else {
            Ok(None)
        }
    }
}

struct PySetAsSequence<'py> {
    iter: Bound<'py, PyIterator>,
}

impl<'py> PySetAsSequence<'py> {
    fn from_set(set: &Bound<'py, PySet>) -> Self {
        Self {
            iter: PyIterator::from_object(set).expect("set is always iterable"),
        }
    }

    fn from_frozenset(set: &Bound<'py, PyFrozenSet>) -> Self {
        Self {
            iter: PyIterator::from_object(set).expect("frozenset is always iterable"),
        }
    }
}

impl<'de> de::SeqAccess<'de> for PySetAsSequence<'_> {
    type Error = PythonizeError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(item) => seed
                .deserialize(&mut Depythonizer::from_object(&item?))
                .map(Some),
            None => Ok(None),
        }
    }
}

struct PyMappingAccess<'py> {
    keys: Bound<'py, PyList>,
    values: Bound<'py, PyList>,
    key_idx: usize,
    val_idx: usize,
    len: usize,
}

impl<'py> PyMappingAccess<'py> {
    fn new(map: &Bound<'py, PyMapping>) -> Result<Self> {
        let keys = map.keys()?;
        let values = map.values()?;
        let len = map.len()?;
        Ok(Self {
            keys,
            values,
            key_idx: 0,
            val_idx: 0,
            len,
        })
    }
}

impl<'de> de::MapAccess<'de> for PyMappingAccess<'_> {
    type Error = PythonizeError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.key_idx < self.len {
            let item = self.keys.get_item(self.key_idx)?;
            self.key_idx += 1;
            seed.deserialize(&mut Depythonizer::from_object(&item))
                .map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        let item = self.values.get_item(self.val_idx)?;
        self.val_idx += 1;
        seed.deserialize(&mut Depythonizer::from_object(&item))
    }
}

struct PyEnumAccess<'a, 'py> {
    de: Depythonizer<'a, 'py>,
    variant: Bound<'py, PyString>,
}

impl<'a, 'py> PyEnumAccess<'a, 'py> {
    fn new(obj: &'a Bound<'py, PyAny>, variant: Bound<'py, PyString>) -> Self {
        Self {
            de: Depythonizer::from_object(obj),
            variant,
        }
    }
}

impl<'de> de::EnumAccess<'de> for PyEnumAccess<'_, '_> {
    type Error = PythonizeError;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        let cow = self.variant.to_cow()?;
        let de: de::value::StrDeserializer<'_, PythonizeError> = cow.as_ref().into_deserializer();
        let val = seed.deserialize(de)?;
        Ok((val, self))
    }
}

impl<'de> de::VariantAccess<'de> for PyEnumAccess<'_, '_> {
    type Error = PythonizeError;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut { self.de })
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(self.de.sequence_access(Some(len))?)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(self.de.dict_access()?)
    }
}

#[cfg(test)]
mod test {
    use std::ffi::CStr;

    use super::*;
    use crate::error::ErrorImpl;
    use maplit::hashmap;
    use pyo3::ffi::c_str;
    use pyo3::{IntoPyObject, Python};
    use serde_json::{json, Value as JsonValue};

    fn test_de<T>(code: &CStr, expected: &T, expected_json: &JsonValue)
    where
        T: de::DeserializeOwned + PartialEq + std::fmt::Debug,
    {
        Python::with_gil(|py| {
            let obj = py.eval(code, None, None).unwrap();
            let actual: T = depythonize(&obj).unwrap();
            assert_eq!(&actual, expected);
            let actual_json: JsonValue = depythonize(&obj).unwrap();
            assert_eq!(&actual_json, expected_json);

            #[allow(deprecated)]
            let actual: T = depythonize_bound(obj.clone()).unwrap();
            assert_eq!(&actual, expected);
        });
    }

    #[test]
    fn test_empty_struct() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Empty;

        let expected = Empty;
        let expected_json = json!(null);
        let code = c_str!("None");
        test_de(code, &expected, &expected_json);
    }

    #[test]
    fn test_struct() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Struct {
            foo: String,
            bar: usize,
            baz: f32,
            qux: bool,
        }

        let expected = Struct {
            foo: "Foo".to_string(),
            bar: 8usize,
            baz: 45.23,
            qux: true,
        };
        let expected_json = json!({
            "foo": "Foo",
            "bar": 8,
            "baz": 45.23,
            "qux": true
        });
        let code = c_str!("{'foo': 'Foo', 'bar': 8, 'baz': 45.23, 'qux': True}");
        test_de(code, &expected, &expected_json);
    }

    #[test]
    fn test_struct_missing_key() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Struct {
            foo: String,
            bar: usize,
        }

        let code = c_str!("{'foo': 'Foo'}");

        Python::with_gil(|py| {
            let locals = PyDict::new(py);
            let obj = py.eval(code, None, Some(&locals)).unwrap();
            assert!(matches!(
                *depythonize::<Struct>(&obj).unwrap_err().inner,
                ErrorImpl::Message(msg) if msg == "missing field `bar`"
            ));
        })
    }

    #[test]
    fn test_tuple_struct() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct TupleStruct(String, f64);

        let expected = TupleStruct("cat".to_string(), -10.05);
        let expected_json = json!(["cat", -10.05]);
        let code = c_str!("('cat', -10.05)");
        test_de(code, &expected, &expected_json);
    }

    #[test]
    fn test_tuple_too_long() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct TupleStruct(String, f64);

        let code = c_str!("('cat', -10.05, 'foo')");

        Python::with_gil(|py| {
            let locals = PyDict::new(py);
            let obj = py.eval(code, None, Some(&locals)).unwrap();
            assert!(matches!(
                *depythonize::<TupleStruct>(&obj).unwrap_err().inner,
                ErrorImpl::IncorrectSequenceLength { expected, got } if expected == 2 && got == 3
            ));
        })
    }

    #[test]
    fn test_tuple_struct_from_pylist() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct TupleStruct(String, f64);

        let expected = TupleStruct("cat".to_string(), -10.05);
        let expected_json = json!(["cat", -10.05]);
        let code = c_str!("['cat', -10.05]");
        test_de(code, &expected, &expected_json);
    }

    #[test]
    fn test_tuple() {
        let expected = ("foo".to_string(), 5);
        let expected_json = json!(["foo", 5]);
        let code = c_str!("('foo', 5)");
        test_de(code, &expected, &expected_json);
    }

    #[test]
    fn test_tuple_from_pylist() {
        let expected = ("foo".to_string(), 5);
        let expected_json = json!(["foo", 5]);
        let code = c_str!("['foo', 5]");
        test_de(code, &expected, &expected_json);
    }

    #[test]
    fn test_vec_from_pyset() {
        let expected = vec!["foo".to_string()];
        let expected_json = json!(["foo"]);
        let code = c_str!("{'foo'}");
        test_de(code, &expected, &expected_json);
    }

    #[test]
    fn test_vec_from_pyfrozenset() {
        let expected = vec!["foo".to_string()];
        let expected_json = json!(["foo"]);
        let code = c_str!("frozenset({'foo'})");
        test_de(code, &expected, &expected_json);
    }

    #[test]
    fn test_vec() {
        let expected = vec![3, 2, 1];
        let expected_json = json!([3, 2, 1]);
        let code = c_str!("[3, 2, 1]");
        test_de(code, &expected, &expected_json);
    }

    #[test]
    fn test_vec_from_tuple() {
        let expected = vec![3, 2, 1];
        let expected_json = json!([3, 2, 1]);
        let code = c_str!("(3, 2, 1)");
        test_de(code, &expected, &expected_json);
    }

    #[test]
    fn test_hashmap() {
        let expected = hashmap! {"foo".to_string() => 4};
        let expected_json = json!({"foo": 4 });
        let code = c_str!("{'foo': 4}");
        test_de(code, &expected, &expected_json);
    }

    #[test]
    fn test_enum_variant() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum Foo {
            Variant,
        }

        let expected = Foo::Variant;
        let expected_json = json!("Variant");
        let code = c_str!("'Variant'");
        test_de(code, &expected, &expected_json);
    }

    #[test]
    fn test_enum_tuple_variant() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum Foo {
            Tuple(i32, String),
        }

        let expected = Foo::Tuple(12, "cat".to_string());
        let expected_json = json!({"Tuple": [12, "cat"]});
        let code = c_str!("{'Tuple': [12, 'cat']}");
        test_de(code, &expected, &expected_json);
    }

    #[test]
    fn test_enum_newtype_variant() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum Foo {
            NewType(String),
        }

        let expected = Foo::NewType("cat".to_string());
        let expected_json = json!({"NewType": "cat" });
        let code = c_str!("{'NewType': 'cat'}");
        test_de(code, &expected, &expected_json);
    }

    #[test]
    fn test_enum_struct_variant() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum Foo {
            Struct { foo: String, bar: usize },
        }

        let expected = Foo::Struct {
            foo: "cat".to_string(),
            bar: 25,
        };
        let expected_json = json!({"Struct": {"foo": "cat", "bar": 25 }});
        let code = c_str!("{'Struct': {'foo': 'cat', 'bar': 25}}");
        test_de(code, &expected, &expected_json);
    }
    #[test]
    fn test_enum_untagged_tuple_variant() {
        #[derive(Debug, Deserialize, PartialEq)]
        #[serde(untagged)]
        enum Foo {
            Tuple(f32, char),
        }

        let expected = Foo::Tuple(12.0, 'c');
        let expected_json = json!([12.0, 'c']);
        let code = c_str!("[12.0, 'c']");
        test_de(code, &expected, &expected_json);
    }

    #[test]
    fn test_enum_untagged_newtype_variant() {
        #[derive(Debug, Deserialize, PartialEq)]
        #[serde(untagged)]
        enum Foo {
            NewType(String),
        }

        let expected = Foo::NewType("cat".to_string());
        let expected_json = json!("cat");
        let code = c_str!("'cat'");
        test_de(code, &expected, &expected_json);
    }

    #[test]
    fn test_enum_untagged_struct_variant() {
        #[derive(Debug, Deserialize, PartialEq)]
        #[serde(untagged)]
        enum Foo {
            Struct { foo: Vec<char>, bar: [u8; 4] },
        }

        let expected = Foo::Struct {
            foo: vec!['a', 'b', 'c'],
            bar: [2, 5, 3, 1],
        };
        let expected_json = json!({"foo": ["a", "b", "c"], "bar": [2, 5, 3, 1]});
        let code = c_str!("{'foo': ['a', 'b', 'c'], 'bar': [2, 5, 3, 1]}");
        test_de(code, &expected, &expected_json);
    }

    #[test]
    fn test_nested_type() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Foo {
            name: String,
            bar: Bar,
        }

        #[derive(Debug, Deserialize, PartialEq)]
        struct Bar {
            value: usize,
            variant: Baz,
        }

        #[derive(Debug, Deserialize, PartialEq)]
        enum Baz {
            Basic,
            Tuple(f32, u32),
        }

        let expected = Foo {
            name: "SomeFoo".to_string(),
            bar: Bar {
                value: 13,
                variant: Baz::Tuple(-1.5, 8),
            },
        };
        let expected_json =
            json!({"name": "SomeFoo", "bar": { "value": 13, "variant": { "Tuple": [-1.5, 8]}}});
        let code =
            c_str!("{'name': 'SomeFoo', 'bar': {'value': 13, 'variant': {'Tuple': [-1.5, 8]}}}");
        test_de(code, &expected, &expected_json);
    }

    #[test]
    fn test_int_limits() {
        Python::with_gil(|py| {
            // serde_json::Value supports u64 and i64 as maxiumum sizes
            let _: serde_json::Value = depythonize(&u8::MAX.into_pyobject(py).unwrap()).unwrap();
            let _: serde_json::Value = depythonize(&u8::MIN.into_pyobject(py).unwrap()).unwrap();
            let _: serde_json::Value = depythonize(&i8::MAX.into_pyobject(py).unwrap()).unwrap();
            let _: serde_json::Value = depythonize(&i8::MIN.into_pyobject(py).unwrap()).unwrap();

            let _: serde_json::Value = depythonize(&u16::MAX.into_pyobject(py).unwrap()).unwrap();
            let _: serde_json::Value = depythonize(&u16::MIN.into_pyobject(py).unwrap()).unwrap();
            let _: serde_json::Value = depythonize(&i16::MAX.into_pyobject(py).unwrap()).unwrap();
            let _: serde_json::Value = depythonize(&i16::MIN.into_pyobject(py).unwrap()).unwrap();

            let _: serde_json::Value = depythonize(&u32::MAX.into_pyobject(py).unwrap()).unwrap();
            let _: serde_json::Value = depythonize(&u32::MIN.into_pyobject(py).unwrap()).unwrap();
            let _: serde_json::Value = depythonize(&i32::MAX.into_pyobject(py).unwrap()).unwrap();
            let _: serde_json::Value = depythonize(&i32::MIN.into_pyobject(py).unwrap()).unwrap();

            let _: serde_json::Value = depythonize(&u64::MAX.into_pyobject(py).unwrap()).unwrap();
            let _: serde_json::Value = depythonize(&u64::MIN.into_pyobject(py).unwrap()).unwrap();
            let _: serde_json::Value = depythonize(&i64::MAX.into_pyobject(py).unwrap()).unwrap();
            let _: serde_json::Value = depythonize(&i64::MIN.into_pyobject(py).unwrap()).unwrap();

            let _: u128 = depythonize(&u128::MAX.into_pyobject(py).unwrap()).unwrap();
            let _: i128 = depythonize(&u128::MIN.into_pyobject(py).unwrap()).unwrap();

            let _: i128 = depythonize(&i128::MAX.into_pyobject(py).unwrap()).unwrap();
            let _: i128 = depythonize(&i128::MIN.into_pyobject(py).unwrap()).unwrap();
        });
    }

    #[test]
    fn test_deserialize_bytes() {
        Python::with_gil(|py| {
            let obj = PyBytes::new(py, "hello".as_bytes());
            let actual: Vec<u8> = depythonize(&obj).unwrap();
            assert_eq!(actual, b"hello");
        })
    }

    #[test]
    fn test_char() {
        let expected = 'a';
        let expected_json = json!("a");
        let code = c_str!("'a'");
        test_de(code, &expected, &expected_json);
    }

    #[test]
    fn test_unknown_type() {
        Python::with_gil(|py| {
            let obj = py
                .import("decimal")
                .unwrap()
                .getattr("Decimal")
                .unwrap()
                .call0()
                .unwrap();
            let err = depythonize::<serde_json::Value>(&obj).unwrap_err();
            assert!(matches!(
                *err.inner,
                ErrorImpl::UnsupportedType(name) if name == "Decimal"
            ));
        });
    }
}
