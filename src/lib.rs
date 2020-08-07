use pyo3::types::{PyDict, PyList};
use pyo3::{IntoPy, Py, PyErr, PyObject, PyResult, Python, PyNativeType};
use serde::{ser, Serialize, Serializer};

pub fn pythonize<T: Serialize>(py: Python, value: T) -> PyResult<PyObject> {
    Ok(value.serialize(Pythonizer { py }).unwrap())
}

pub fn depythonize<T>(py: Python, obj: PyObject) -> T {
    todo!()
}

#[derive(Debug)]
pub struct DummyErr {}

impl ser::Error for DummyErr {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        todo!()
    }
}

impl std::fmt::Display for DummyErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for DummyErr {}

#[derive(Copy, Clone)]
pub struct Pythonizer<'py> {
    py: Python<'py>,
}

pub struct PythonDictSerializer<'py> {
    dict: &'py PyDict,
}

impl<'py> Serializer for Pythonizer<'py> {
    type Ok = PyObject;
    type Error = DummyErr;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = PythonDictSerializer<'py>;
    type SerializeStructVariant = Self;
    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(v.into_py(self.py))
    }
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(v.into_py(self.py))
    }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(v.into_py(self.py))
    }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(v.into_py(self.py))
    }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(v.into_py(self.py))
    }
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(v.into_py(self.py))
    }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(v.into_py(self.py))
    }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(v.into_py(self.py))
    }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(v.into_py(self.py))
    }
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(v.into_py(self.py))
    }
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(v.into_py(self.py))
    }
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(v.into_py(self.py))
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(v.into_py(self.py))
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.py.None())
    }
    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.py.None())
    }
    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(self.py.None())
    }
    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(self.py.None())
    }
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        todo!()
    }
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        todo!()
    }
    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        todo!()
    }
    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        todo!()
    }
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        todo!()
    }
    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(PythonDictSerializer {
            dict: PyDict::new(self.py),
        })
    }
    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        todo!()
    }
}

impl ser::SerializeTupleVariant for Pythonizer<'_> {
    type Ok = PyObject;
    type Error = DummyErr;
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        todo!()
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl ser::SerializeTuple for Pythonizer<'_> {
    type Ok = PyObject;
    type Error = DummyErr;
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        todo!()
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl ser::SerializeTupleStruct for Pythonizer<'_> {
    type Ok = PyObject;
    type Error = DummyErr;
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        todo!()
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl ser::SerializeStruct for PythonDictSerializer<'_> {
    type Ok = PyObject;
    type Error = DummyErr;
    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        Ok(self.dict.set_item(key, pythonize(self.dict.py(), value).unwrap()).unwrap())
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.dict.into())
    }
}

impl ser::SerializeStructVariant for Pythonizer<'_> {
    type Ok = PyObject;
    type Error = DummyErr;
    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        todo!()
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl ser::SerializeMap for Pythonizer<'_> {
    type Ok = PyObject;
    type Error = DummyErr;
    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        todo!()
    }
    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        todo!()
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl ser::SerializeSeq for Pythonizer<'_> {
    type Ok = PyObject;
    type Error = DummyErr;
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        todo!()
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

pub struct Depythonizer;

#[cfg(test)]
mod test {
    use super::pythonize;
    use pyo3::types::PyDict;
    use pyo3::{PyResult, Python};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct SampleDict {
        foo: String,
        bar: usize,
    }

    #[test]
    fn test_sample_dict() {
        Python::with_gil(|py| -> PyResult<()> {
            let s = SampleDict {
                foo: "foo".to_string(),
                bar: 5,
            };

            let obj = pythonize(py, s)?;
            let d: &PyDict = obj.extract(py)?;

            assert_eq!(d.get_item("foo").unwrap().extract::<&str>()?, "foo");

            Ok(())
        })
        .unwrap();
    }
}
