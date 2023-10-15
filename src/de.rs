use pyo3::types::*;
use serde::de::{self, IntoDeserializer};
use serde::Deserialize;

use crate::error::{PythonizeError, Result};

/// Attempt to convert a Python object to an instance of `T`
pub fn depythonize<'de, T>(obj: &'de PyAny) -> Result<T>
where
    T: Deserialize<'de>,
{
    let mut depythonizer = Depythonizer::from_object(obj);
    T::deserialize(&mut depythonizer)
}

pub struct Depythonizer<'de> {
    input: &'de PyAny,
}

impl<'de> Depythonizer<'de> {
    pub fn from_object(input: &'de PyAny) -> Self {
        Depythonizer { input }
    }

    fn sequence_access(&self, expected_len: Option<usize>) -> Result<PySequenceAccess<'de>> {
        let seq: &PySequence = self.input.downcast()?;
        let len = self.input.len()?;

        match expected_len {
            Some(expected) if expected != len => {
                Err(PythonizeError::incorrect_sequence_length(expected, len))
            }
            _ => Ok(PySequenceAccess::new(seq, len)),
        }
    }

    fn dict_access(&self) -> Result<PyMappingAccess<'de>> {
        PyMappingAccess::new(self.input.downcast()?)
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

impl<'a, 'de> de::Deserializer<'de> for &'a mut Depythonizer<'de> {
    type Error = PythonizeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let obj = self.input;

        if obj.is_none() {
            self.deserialize_unit(visitor)
        } else if obj.is_instance_of::<PyBool>() {
            self.deserialize_bool(visitor)
        } else if obj.is_instance_of::<PyByteArray>() || obj.is_instance_of::<PyBytes>() {
            self.deserialize_bytes(visitor)
        } else if obj.is_instance_of::<PyDict>() {
            self.deserialize_map(visitor)
        } else if obj.is_instance_of::<PyFloat>() {
            self.deserialize_f64(visitor)
        } else if obj.is_instance_of::<PyFrozenSet>() {
            self.deserialize_tuple(obj.len()?, visitor)
        } else if obj.is_instance_of::<PyInt>() {
            self.deserialize_i64(visitor)
        } else if obj.is_instance_of::<PyList>() {
            self.deserialize_tuple(obj.len()?, visitor)
        } else if obj.is_instance_of::<PyLong>() {
            self.deserialize_i64(visitor)
        } else if obj.is_instance_of::<PySet>() {
            self.deserialize_tuple(obj.len()?, visitor)
        } else if obj.is_instance_of::<PyString>() {
            self.deserialize_str(visitor)
        } else if obj.is_instance_of::<PyTuple>() {
            self.deserialize_tuple(obj.len()?, visitor)
        } else if obj.is_instance_of::<PyUnicode>() {
            self.deserialize_str(visitor)
        } else if obj.downcast::<PySequence>().is_ok() {
            self.deserialize_tuple(obj.len()?, visitor)
        } else if obj.downcast::<PyMapping>().is_ok() {
            self.deserialize_map(visitor)
        } else {
            Err(PythonizeError::unsupported_type(
                obj.get_type().name().unwrap_or("<unknown>"),
            ))
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_bool(self.input.is_true()?)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let s = self.input.downcast::<PyString>()?.to_str()?;
        if s.len() != 1 {
            return Err(PythonizeError::invalid_length_char());
        }
        visitor.visit_char(s.chars().next().unwrap())
    }

    deserialize_type!(deserialize_i8 => visit_i8);
    deserialize_type!(deserialize_i16 => visit_i16);
    deserialize_type!(deserialize_i32 => visit_i32);
    deserialize_type!(deserialize_i64 => visit_i64);
    deserialize_type!(deserialize_u8 => visit_u8);
    deserialize_type!(deserialize_u16 => visit_u16);
    deserialize_type!(deserialize_u32 => visit_u32);
    deserialize_type!(deserialize_u64 => visit_u64);
    deserialize_type!(deserialize_f32 => visit_f32);
    deserialize_type!(deserialize_f64 => visit_f64);

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let s: &PyString = self.input.downcast()?;
        visitor.visit_str(s.to_str()?)
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
        let obj = self.input;
        let b: &PyBytes = obj.downcast()?;
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
        visitor.visit_seq(self.sequence_access(None)?)
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
        let item = self.input;
        if item.is_instance_of::<PyDict>() {
            // Get the enum variant from the dict key
            let d: &PyDict = item.downcast().unwrap();
            if d.len() != 1 {
                return Err(PythonizeError::invalid_length_enum());
            }
            let variant: &PyString = d
                .keys()
                .get_item(0)?
                .downcast()
                .map_err(|_| PythonizeError::dict_key_not_string())?;
            let value = d.get_item(variant)?.unwrap();
            let mut de = Depythonizer::from_object(value);
            visitor.visit_enum(PyEnumAccess::new(&mut de, variant))
        } else if item.is_instance_of::<PyString>() {
            let s: &PyString = self.input.downcast()?;
            visitor.visit_enum(s.to_str()?.into_deserializer())
        } else {
            Err(PythonizeError::invalid_enum_type())
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let s: &PyString = self
            .input
            .downcast()
            .map_err(|_| PythonizeError::dict_key_not_string())?;
        visitor.visit_str(s.to_str()?)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_unit()
    }
}

struct PySequenceAccess<'a> {
    seq: &'a PySequence,
    index: usize,
    len: usize,
}

impl<'a> PySequenceAccess<'a> {
    fn new(seq: &'a PySequence, len: usize) -> Self {
        Self { seq, index: 0, len }
    }
}

impl<'de> de::SeqAccess<'de> for PySequenceAccess<'de> {
    type Error = PythonizeError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.index < self.len {
            let mut item_de = Depythonizer::from_object(self.seq.get_item(self.index)?);
            self.index += 1;
            seed.deserialize(&mut item_de).map(Some)
        } else {
            Ok(None)
        }
    }
}

struct PyMappingAccess<'de> {
    keys: &'de PySequence,
    values: &'de PySequence,
    key_idx: usize,
    val_idx: usize,
    len: usize,
}

impl<'de> PyMappingAccess<'de> {
    fn new(map: &'de PyMapping) -> Result<Self> {
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

impl<'de> de::MapAccess<'de> for PyMappingAccess<'de> {
    type Error = PythonizeError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.key_idx < self.len {
            let mut item_de = Depythonizer::from_object(self.keys.get_item(self.key_idx)?);
            self.key_idx += 1;
            seed.deserialize(&mut item_de).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        let mut item_de = Depythonizer::from_object(self.values.get_item(self.val_idx)?);
        self.val_idx += 1;
        seed.deserialize(&mut item_de)
    }
}

struct PyEnumAccess<'a, 'de> {
    de: &'a mut Depythonizer<'de>,
    variant: &'de PyString,
}

impl<'a, 'de> PyEnumAccess<'a, 'de> {
    fn new(de: &'a mut Depythonizer<'de>, variant: &'de PyString) -> Self {
        Self { de, variant }
    }
}

impl<'a, 'de> de::EnumAccess<'de> for PyEnumAccess<'a, 'de> {
    type Error = PythonizeError;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        let de: de::value::StrDeserializer<'_, PythonizeError> =
            self.variant.to_str()?.into_deserializer();
        let val = seed.deserialize(de)?;
        Ok((val, self))
    }
}

impl<'a, 'de> de::VariantAccess<'de> for PyEnumAccess<'a, 'de> {
    type Error = PythonizeError;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
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
    use super::*;
    use crate::error::ErrorImpl;
    use maplit::hashmap;
    use pyo3::Python;
    use serde_json::{json, Value as JsonValue};

    fn test_de<T>(code: &str, expected: &T, expected_json: &JsonValue)
    where
        T: de::DeserializeOwned + PartialEq + std::fmt::Debug,
    {
        Python::with_gil(|py| {
            let locals = PyDict::new(py);
            py.run(&format!("obj = {}", code), None, Some(locals))
                .unwrap();
            let obj = locals.get_item("obj").unwrap().unwrap();
            let actual: T = depythonize(obj).unwrap();
            assert_eq!(&actual, expected);
            let actual_json: JsonValue = depythonize(obj).unwrap();
            assert_eq!(&actual_json, expected_json);
        });
    }

    #[test]
    fn test_empty_struct() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Empty;

        let expected = Empty;
        let expected_json = json!(null);
        let code = "None";
        test_de(code, &expected, &expected_json);
    }

    #[test]
    fn test_struct() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Struct {
            foo: String,
            bar: usize,
            baz: f32,
        }

        let expected = Struct {
            foo: "Foo".to_string(),
            bar: 8usize,
            baz: 45.23,
        };
        let expected_json = json!({
            "foo": "Foo",
            "bar": 8,
            "baz": 45.23
        });
        let code = "{'foo': 'Foo', 'bar': 8, 'baz': 45.23}";
        test_de(code, &expected, &expected_json);
    }

    #[test]
    fn test_struct_missing_key() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Struct {
            foo: String,
            bar: usize,
        }

        let code = "{'foo': 'Foo'}";

        Python::with_gil(|py| {
            let locals = PyDict::new(py);
            py.run(&format!("obj = {}", code), None, Some(locals))
                .unwrap();
            let obj = locals.get_item("obj").unwrap().unwrap();
            assert!(matches!(
                *depythonize::<Struct>(obj).unwrap_err().inner,
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
        let code = "('cat', -10.05)";
        test_de(code, &expected, &expected_json);
    }

    #[test]
    fn test_tuple_too_long() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct TupleStruct(String, f64);

        let code = "('cat', -10.05, 'foo')";

        Python::with_gil(|py| {
            let locals = PyDict::new(py);
            py.run(&format!("obj = {}", code), None, Some(locals))
                .unwrap();
            let obj = locals.get_item("obj").unwrap().unwrap();
            assert!(matches!(
                *depythonize::<TupleStruct>(obj).unwrap_err().inner,
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
        let code = "['cat', -10.05]";
        test_de(code, &expected, &expected_json);
    }

    #[test]
    fn test_tuple() {
        let expected = ("foo".to_string(), 5);
        let expected_json = json!(["foo", 5]);
        let code = "('foo', 5)";
        test_de(code, &expected, &expected_json);
    }

    #[test]
    fn test_tuple_from_pylist() {
        let expected = ("foo".to_string(), 5);
        let expected_json = json!(["foo", 5]);
        let code = "['foo', 5]";
        test_de(code, &expected, &expected_json);
    }

    #[test]
    fn test_vec() {
        let expected = vec![3, 2, 1];
        let expected_json = json!([3, 2, 1]);
        let code = "[3, 2, 1]";
        test_de(code, &expected, &expected_json);
    }

    #[test]
    fn test_vec_from_tuple() {
        let expected = vec![3, 2, 1];
        let expected_json = json!([3, 2, 1]);
        let code = "(3, 2, 1)";
        test_de(code, &expected, &expected_json);
    }

    #[test]
    fn test_hashmap() {
        let expected = hashmap! {"foo".to_string() => 4};
        let expected_json = json!({"foo": 4 });
        let code = "{'foo': 4}";
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
        let code = "'Variant'";
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
        let code = "{'Tuple': [12, 'cat']}";
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
        let code = "{'NewType': 'cat'}";
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
        let code = "{'Struct': {'foo': 'cat', 'bar': 25}}";
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
        let code = "[12.0, 'c']";
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
        let code = "'cat'";
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
        let code = "{'foo': ['a', 'b', 'c'], 'bar': [2, 5, 3, 1]}";
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
        let code = "{'name': 'SomeFoo', 'bar': {'value': 13, 'variant': {'Tuple': [-1.5, 8]}}}";
        test_de(code, &expected, &expected_json);
    }
}
