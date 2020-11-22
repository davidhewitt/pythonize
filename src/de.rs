use pyo3::type_object::PyTypeInfo;
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

#[derive(Debug)]
enum GetItemKey<'de> {
    Key(&'de PyAny),
    Index(isize),
    None,
}

pub struct Depythonizer<'de> {
    input: &'de PyAny,
    // Denotes the current dict key or collection index we are deserializing
    current: GetItemKey<'de>,
    // Used to indicate the current string being deserialized is a dict key not value
    as_key: bool,
}

impl<'de> Depythonizer<'de> {
    pub fn from_object(input: &'de PyAny) -> Self {
        Depythonizer {
            input,
            current: GetItemKey::None,
            as_key: false,
        }
    }

    fn get_item(&self) -> Result<Option<&'de PyAny>> {
        match self.current {
            GetItemKey::Key(k) => {
                let dict: &PyDict = self.input.cast_as()?;
                Ok(dict.get_item(&k).map(|obj| obj.into()))
            }
            GetItemKey::Index(i) => {
                let list: &PyList = self.input.cast_as()?;
                let len = list.len() as isize;
                if i >= len {
                    Ok(None)
                } else if i < -len {
                    Ok(None)
                } else {
                    Ok(Some(list.get_item(i).into()))
                }
            }
            GetItemKey::None => Ok(Some(self.input.clone())),
        }
    }

    fn get_item_or_missing(&self) -> Result<&'de PyAny> {
        self.get_item()?
            .ok_or_else(|| PythonizeError::missing(&self.current))
    }

    fn get_dict_keys(&self) -> Result<Vec<&'de PyAny>> {
        let d: &PyDict = self.get_item_or_missing()?.cast_as()?;
        Ok(d.keys().iter().collect())
    }
}

macro_rules! deserialize_type {
    ($method:ident => $visit:ident) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value>
        where
            V: de::Visitor<'de>,
        {
            visitor.$visit(self.get_item_or_missing()?.extract()?)
        }
    };
}

impl<'a, 'de> de::Deserializer<'de> for &'a mut Depythonizer<'de> {
    type Error = PythonizeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let obj = self.get_item_or_missing()?;

        if obj.is_none() {
            self.deserialize_unit(visitor)
        } else if PyBool::is_instance(obj) {
            self.deserialize_bool(visitor)
        } else if PyByteArray::is_instance(obj) {
            self.deserialize_bytes(visitor)
        } else if PyBytes::is_instance(obj) {
            self.deserialize_bytes(visitor)
        } else if PyDict::is_instance(obj) {
            self.deserialize_map(visitor)
        } else if PyFloat::is_instance(obj) {
            self.deserialize_f64(visitor)
        } else if PyFrozenSet::is_instance(obj) {
            self.deserialize_tuple(obj.len()?, visitor)
        } else if PyInt::is_instance(obj) {
            self.deserialize_i64(visitor)
        } else if PyList::is_instance(obj) {
            self.deserialize_tuple(obj.len()?, visitor)
        } else if PyLong::is_instance(obj) {
            self.deserialize_i64(visitor)
        } else if PySet::is_instance(obj) {
            self.deserialize_tuple(obj.len()?, visitor)
        } else if PyString::is_instance(obj) {
            self.deserialize_str(visitor)
        } else if PyTuple::is_instance(obj) {
            self.deserialize_tuple(obj.len()?, visitor)
        } else if PyUnicode::is_instance(obj) {
            self.deserialize_str(visitor)
        } else {
            Err(PythonizeError::unsupported_type(obj.get_type().name()))
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_bool(self.get_item_or_missing()?.is_true()?)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let s = self
            .get_item_or_missing()?
            .cast_as::<PyString>()?
            .to_str()?;
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
        if self.as_key {
            match self.current {
                GetItemKey::Key(key) => {
                    let k: &PyString = key.cast_as()?;
                    visitor.visit_str(k.to_str()?)
                }
                _ => visitor.visit_str(""),
            }
        } else {
            let s: &PyString = self.get_item_or_missing()?.cast_as()?;
            visitor.visit_str(s.to_str()?)
        }
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
        let obj = self.get_item_or_missing()?;
        let b: &PyBytes = obj.cast_as()?;
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
        if self.get_item_or_missing()?.is_none() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.get_item_or_missing()?.is_none() {
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
        let seq = self.get_item_or_missing()?;
        let mut dep = Depythonizer::from_object(seq);
        visitor.visit_seq(PyListAccess::new(&mut dep))
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.current {
            GetItemKey::None => {
                let keys = self.get_dict_keys()?;
                visitor.visit_map(PyDictAccess::new(self, keys))
            }
            _ => {
                let obj = self.get_item_or_missing()?;
                let keys = self.get_dict_keys()?;
                let mut dep = Depythonizer::from_object(obj);
                visitor.visit_map(PyDictAccess::new(&mut dep, keys))
            }
        }
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
        let item = self.get_item_or_missing()?;
        if PyDict::is_instance(item) {
            // Get the enum variant from the dict key
            let d: &PyDict = item.cast_as().unwrap();
            if d.len() != 1 {
                return Err(PythonizeError::invalid_length_enum());
            }
            let variant: &PyString = d
                .keys()
                .get_item(0)
                .cast_as()
                .map_err(|_| PythonizeError::dict_key_not_string())?;
            let value = d.get_item(variant).unwrap();
            let mut de = Depythonizer::from_object(value);
            visitor.visit_enum(PyEnumAccess::new(&mut de, variant))
        } else if PyString::is_instance(item) {
            let s: &PyString = self.input.cast_as()?;
            visitor.visit_enum(s.to_str()?.into_deserializer())
        } else {
            Err(PythonizeError::invalid_enum_type())
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match &self.current {
            // Deserialize map key
            GetItemKey::Key(obj) => {
                let s: &PyString = obj
                    .cast_as()
                    .map_err(|_| PythonizeError::dict_key_not_string())?;
                visitor.visit_str(s.to_str()?)
            }
            // Deserialize externally-tagged enum variant
            _ => Err(PythonizeError::dict_key_not_string()),
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_unit()
    }
}

struct PyListAccess<'a, 'de> {
    de: &'a mut Depythonizer<'de>,
    index: isize,
}

impl<'a, 'de> PyListAccess<'a, 'de> {
    fn new(de: &'a mut Depythonizer<'de>) -> Self {
        Self { de, index: 0 }
    }
}

impl<'a, 'de> de::SeqAccess<'de> for PyListAccess<'a, 'de> {
    type Error = PythonizeError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        self.de.current = GetItemKey::Index(self.index);
        self.index += 1;
        match self.de.get_item()? {
            Some(_obj) => seed.deserialize(&mut *self.de).map(Some),
            None => Ok(None),
        }
    }
}

struct PyDictAccess<'a, 'de> {
    de: &'a mut Depythonizer<'de>,
    keys: Vec<&'de PyAny>,
    index: usize,
}

impl<'a, 'de> PyDictAccess<'a, 'de> {
    fn new(de: &'a mut Depythonizer<'de>, keys: Vec<&'de PyAny>) -> Self {
        Self { de, keys, index: 0 }
    }
}

impl<'a, 'de> de::MapAccess<'de> for PyDictAccess<'a, 'de> {
    type Error = PythonizeError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.index >= self.keys.len() {
            Ok(None)
        } else {
            let key = self.keys[self.index];
            self.de.current = GetItemKey::Key(key);
            self.de.as_key = true;
            self.index += 1;
            seed.deserialize(&mut *self.de).map(Some)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        self.de.as_key = false;
        seed.deserialize(&mut *self.de)
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

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(PyListAccess::new(self.de))
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(PyDictAccess::new(self.de, self.de.get_dict_keys()?))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use maplit::hashmap;
    use pyo3::Python;

    fn test_de<T>(code: &str, expected: &T)
    where
        T: de::DeserializeOwned + PartialEq + std::fmt::Debug,
    {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let locals = PyDict::new(py);
        py.run(&format!("obj = {}", code), None, Some(locals))
            .unwrap();
        let obj = locals.get_item("obj").unwrap();
        let actual: T = depythonize(obj).unwrap();
        assert_eq!(&actual, expected);
    }

    #[test]
    fn test_empty_struct() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Empty;

        let expected = Empty;
        let code = "None";
        test_de(code, &expected);
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
        let code = "{'foo': 'Foo', 'bar': 8, 'baz': 45.23}";
        test_de(code, &expected);
    }

    #[test]
    fn test_tuple_struct() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct TupleStruct(String, f64);

        let expected = TupleStruct("cat".to_string(), -10.05);
        let code = "['cat', -10.05]";
        test_de(code, &expected);
    }

    #[test]
    fn test_tuple() {
        let expected = ("foo".to_string(), 5);
        let code = "['foo', 5]";
        test_de(code, &expected);
    }

    #[test]
    fn test_vec() {
        let expected = vec![3, 2, 1];
        let code = "[3, 2, 1]";
        test_de(code, &expected);
    }

    #[test]
    fn test_hashmap() {
        let expected = hashmap! {"foo".to_string() => 4};
        let code = "{'foo': 4}";
        test_de(code, &expected);
    }

    #[test]
    fn test_enum_variant() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum Foo {
            Variant,
        }

        let expected = Foo::Variant;
        let code = "'Variant'";
        test_de(code, &expected);
    }

    #[test]
    fn test_enum_tuple_variant() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum Foo {
            Tuple(i32, String),
        }

        let expected = Foo::Tuple(12, "cat".to_string());
        let code = "{'Tuple': [12, 'cat']}";
        test_de(code, &expected);
    }

    #[test]
    fn test_enum_newtype_variant() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum Foo {
            NewType(String),
        }

        let expected = Foo::NewType("cat".to_string());
        let code = "{'NewType': 'cat'}";
        test_de(code, &expected);
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
        let code = "{'Struct': {'foo': 'cat', 'bar': 25}}";
        test_de(code, &expected);
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
        let code = "{'name': 'SomeFoo', 'bar': {'value': 13, 'variant': {'Tuple': [-1.5, 8]}}}";
        test_de(code, &expected);
    }
}
