/// Pythonize has two public APIs: `pythonize` and `depythonize`.
///
/// ```
/// use serde::{Serialize, Deserialize};
/// use pyo3::{Python, AsPyRef};
/// use pythonize::pythonize;
///
/// #[derive(Serialize, Deserialize)]
/// struct Sample {
///     foo: String,
///     bar: Option<usize>
/// }
///
/// let gil = Python::acquire_gil();
/// let py = gil.python();
///
/// let sample = Sample {
///     foo: "foo".to_string(),
///     bar: None
/// };
///
/// let obj = pythonize(py, &sample).expect("failed to pythonize sample");
///
/// println!("{}", obj.as_ref(py).repr().expect("failed to get repr"));
///
/// // XXX: depythonize is not yet implemented!
/// ```

use pyo3::types::{PyDict, PyList, PyTuple};
use pyo3::{IntoPy, PyErr, PyNativeType, PyObject, PyResult, Python};
use serde::{ser, Serialize, Serializer};

pub fn pythonize<T: Serialize>(py: Python, value: T) -> PyResult<PyObject> {
    Ok(value.serialize(Pythonizer { py })?)
}

pub fn depythonize<T>(_py: Python, _obj: PyObject) -> T {
    todo!()
}

#[derive(Debug)]
pub struct PythonizerError(PyErr);

impl ser::Error for PythonizerError {
    fn custom<T>(_msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        todo!()
    }
}

impl std::fmt::Display for PythonizerError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for PythonizerError {}

impl From<PyErr> for PythonizerError {
    fn from(other: PyErr) -> Self {
        Self(other)
    }
}

impl From<PythonizerError> for PyErr {
    fn from(other: PythonizerError) -> Self {
        other.0
    }
}

#[derive(Clone, Copy)]
pub struct Pythonizer<'py> {
    py: Python<'py>
}

#[doc(hidden)]
pub struct PythonDictSerializer<'py> {
    dict: &'py PyDict,
}

#[doc(hidden)]
pub struct PythonMapSerializer<'py> {
    dict: &'py PyDict,
    key: Option<PyObject>
}


#[doc(hidden)]
pub struct PythonListSerializer<'py> {
    list: &'py PyList,
}

#[doc(hidden)]
pub struct PythonTupleSerializer<'py> {
    items: Vec<PyObject>,
    py: Python<'py>
}

#[doc(hidden)]
pub struct PythonTupleVariantSerializer<'py> {
    variant: &'static str,
    inner: PythonTupleSerializer<'py>
}

#[doc(hidden)]
pub struct PythonStructVariantSerializer<'py> {
    variant: &'static str,
    inner: PythonDictSerializer<'py>
}

impl<'py> Serializer for Pythonizer<'py> {
    type Ok = PyObject;
    type Error = PythonizerError;
    type SerializeSeq = PythonListSerializer<'py>;
    type SerializeTuple = PythonTupleSerializer<'py>;
    type SerializeTupleStruct = PythonTupleSerializer<'py>;
    type SerializeTupleVariant = PythonTupleVariantSerializer<'py>;
    type SerializeMap = PythonMapSerializer<'py>;
    type SerializeStruct = PythonDictSerializer<'py>;
    type SerializeStructVariant = PythonStructVariantSerializer<'py>;
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
    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
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
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(self.py.None())
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(variant.into_py(self.py))
    }
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        let d = PyDict::new(self.py);
        d.set_item(variant, value.serialize(self)?)?;
        Ok(d.into())
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(PythonListSerializer { list: PyList::empty(self.py) })
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(PythonTupleSerializer {
            items: Vec::new(),
            py: self.py
        })
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(PythonTupleSerializer {
            items: Vec::new(),
            py: self.py
        })
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(PythonTupleVariantSerializer {
            variant,
            inner: PythonTupleSerializer {
                items: Vec::new(),
                py: self.py
            }
        })
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(PythonMapSerializer {
            dict: PyDict::new(self.py),
            key: None
        })
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(PythonDictSerializer {
            dict: PyDict::new(self.py),
        })
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(PythonStructVariantSerializer {
            variant,
            inner: PythonDictSerializer {
                dict: PyDict::new(self.py)
            }
        })
    }
}

impl ser::SerializeTupleVariant for PythonTupleVariantSerializer<'_> {
    type Ok = PyObject;
    type Error = PythonizerError;
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        Ok(self.inner.items.push(pythonize(self.inner.py, value)?))
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        let d = PyDict::new(self.inner.py);
        d.set_item(self.variant, PyTuple::new(self.inner.py, self.inner.items))?;
        Ok(d.into())
    }
}

impl ser::SerializeTuple for PythonTupleSerializer<'_> {
    type Ok = PyObject;
    type Error = PythonizerError;
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.items.push(pythonize(self.py, value)?);
        Ok(())
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(PyTuple::new(self.py, self.items).into())
    }
}

impl ser::SerializeTupleStruct for PythonTupleSerializer<'_> {
    type Ok = PyObject;
    type Error = PythonizerError;
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.items.push(pythonize(self.py, value)?);
        Ok(())
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(PyTuple::new(self.py, self.items).into())
    }
}

impl ser::SerializeStruct for PythonDictSerializer<'_> {
    type Ok = PyObject;
    type Error = PythonizerError;
    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        Ok(self
            .dict
            .set_item(key, pythonize(self.dict.py(), value)?)?)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.dict.into())
    }
}

impl ser::SerializeStructVariant for PythonStructVariantSerializer<'_> {
    type Ok = PyObject;
    type Error = PythonizerError;
    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.inner.dict.set_item(key, pythonize(self.inner.dict.py(), value)?)?;
        Ok(())
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        let d = PyDict::new(self.inner.dict.py());
        d.set_item(self.variant, self.inner.dict)?;
        Ok(d.into())
    }
}

impl ser::SerializeMap for PythonMapSerializer<'_> {
    type Ok = PyObject;
    type Error = PythonizerError;
    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.key = Some(pythonize(self.dict.py(), key)?);
        Ok(())
    }
    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.dict.set_item(
            self.key.take().expect("serialize_value should always be called after serialize_key"),
            pythonize(self.dict.py(), value)?
        )?;
        Ok(())
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.dict.into())
    }
}

impl ser::SerializeSeq for PythonListSerializer<'_> {
    type Ok = PyObject;
    type Error = PythonizerError;
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        Ok(self.list.append(pythonize(self.list.py(), value)?)?)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.list.into())
    }
}

pub struct Depythonizer;

#[cfg(test)]
mod test {
    use super::pythonize;
    use maplit::hashmap;
    use paste::paste;
    use pyo3::types::PyDict;
    use pyo3::{PyResult, Python};
    use serde::{Deserialize, Serialize};

    macro_rules! test_sample {
        ($name:ident, $sample:expr, $expected:literal) => {
            paste!(
                #[test]
                fn [<test_sample_ $name>] () -> PyResult<()> {
                    let gil = Python::acquire_gil();
                    let py = gil.python();

                    let sample = $sample;
                    let obj = pythonize(py, &sample)?;

                    let locals = PyDict::new(py);
                    locals.set_item("obj", obj)?;

                    py.run("import json; result = json.dumps(obj, separators=(',', ':'))", None, Some(locals))?;
                    let result = locals.get_item("result").unwrap().extract::<&str>()?;

                    assert_eq!(result, $expected);
                    assert_eq!(serde_json::to_string(&sample).unwrap(), $expected);

                    Ok(())
                }
            );
        };
    }

    test_sample!(
        empty_struct,
        {
            #[derive(Serialize, Deserialize)]
            struct Empty;

            Empty
        },
        r#"null"#
    );

    test_sample!(
        struct,
        {
            #[derive(Serialize, Deserialize)]
            struct Struct {
                foo: String,
                bar: usize,
            }

            Struct {
                foo: "foo".to_string(),
                bar: 5
            }
        },
        r#"{"foo":"foo","bar":5}"#
    );

    test_sample!(
        tuple_struct,
        {
            #[derive(Serialize, Deserialize)]
            struct TupleStruct(String, usize);

            TupleStruct("foo".to_string(), 5)
        },
        r#"["foo",5]"#
    );

    test_sample!(
        tuple,
        ("foo", 5),
        r#"["foo",5]"#
    );

    test_sample!(
        vec,
        vec![1, 2, 3],
        r#"[1,2,3]"#
    );

    test_sample!(
        map,
        hashmap!{"foo" => "foo"},
        r#"{"foo":"foo"}"#
    );

    test_sample!(
        enum_unit_variant,
        {
            #[derive(Serialize, Deserialize)]
            enum E {
                Empty
            }

            E::Empty
        },
        r#""Empty""#
    );

    test_sample!(
        enum_tuple_variant,
        {
            #[derive(Serialize, Deserialize)]
            enum E {
                Tuple(i32, String)
            }

            E::Tuple(5, "foo".to_string())
        },
        r#"{"Tuple":[5,"foo"]}"#
    );

    test_sample!(
        enum_newtype_variant,
        {
            #[derive(Serialize, Deserialize)]
            enum E {
                NewType(String)
            }

            E::NewType("foo".to_string())
        },
        r#"{"NewType":"foo"}"#
    );

    test_sample!(
        enum_struct_variant,
        {
            #[derive(Serialize, Deserialize)]
            enum E {
                Struct { foo: String, bar: usize }
            }

            E::Struct { foo:"foo".to_string(), bar: 5 }
        },
        r#"{"Struct":{"foo":"foo","bar":5}}"#
    );
}
