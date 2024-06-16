use pyo3::{
    types::{PyDict, PyDictMethods, PyList, PyListMethods},
    PyObject, Python, ToPyObject,
};
use serde::{
    de::{MapAccess, SeqAccess, VariantAccess, Visitor},
    Deserialize,
};

pub struct PyObjectVisitor<'py> {
    py: Python<'py>,
}

impl<'py> PyObjectVisitor<'py> {
    pub fn new(py: Python) -> PyObjectVisitor<'_> {
        PyObjectVisitor { py }
    }
}

struct Wrapper {
    inner: PyObject,
}

impl<'de> Deserialize<'de> for Wrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Python::with_gil(|py| {
            deserializer
                .deserialize_any(PyObjectVisitor { py })
                .map(|inner| Wrapper { inner })
        })
    }
}

impl<'de, 'py> Visitor<'de> for PyObjectVisitor<'py> {
    type Value = PyObject;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("any PyObject")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(value.to_object(self.py))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(value.to_object(self.py))
    }

    fn visit_i128<E>(self, value: i128) -> Result<Self::Value, E> {
        Ok(value.to_object(self.py))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(value.to_object(self.py))
    }

    fn visit_u128<E>(self, value: u128) -> Result<Self::Value, E> {
        Ok(value.to_object(self.py))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E> {
        Ok(value.to_object(self.py))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(value.to_object(self.py))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(value.to_object(self.py))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E> {
        Ok(v.to_object(self.py))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(self.py.None())
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wrapper: Wrapper = Deserialize::deserialize(deserializer)?;
        Ok(wrapper.inner)
    }

    fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
        self.visit_none()
    }

    fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let list = PyList::empty_bound(self.py);

        while let Some(wrapper) = visitor.next_element::<Wrapper>()? {
            list.append(wrapper.inner).unwrap();
        }

        Ok(list.to_object(self.py))
    }

    fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
    where
        V: MapAccess<'de>,
    {
        let dict = PyDict::new_bound(self.py);

        while let Some((key, value)) = visitor.next_entry::<Wrapper, Wrapper>()? {
            dict.set_item(key.inner, value.inner).unwrap();
        }

        Ok(dict.to_object(self.py))
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::EnumAccess<'de>,
    {
        let dict = PyDict::new_bound(self.py);
        let (value, variant): (Wrapper, _) = data.variant()?;
        let variant: Wrapper = variant.newtype_variant()?;
        dict.set_item(variant.inner, value.inner).unwrap();

        Ok(dict.to_object(self.py))
    }
}

#[cfg(test)]
mod tests {
    use pyo3::{
        types::{PyAnyMethods, PyStringMethods},
        PyObject, Python,
    };
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    #[derive(Serialize, Deserialize)]
    struct Foo {
        #[serde(with = "crate::pyobject")]
        #[serde(flatten)]
        inner: PyObject,
    }

    #[test]
    fn simple_test() {
        let value = json!({
            "code": 200,
            "success": true,
            "payload": {
                "features": [
                    "serde",
                    "json"
                ],
                "homepage": null
            }
        });

        Python::with_gil(|py| {
            let foo = Foo {
                inner: crate::pythonize(py, &value).unwrap(),
            };
            let serialized = serde_json::to_string(&foo).unwrap();
            let deserialized: Foo = serde_json::from_str(&serialized).unwrap();
            assert_eq!(
                deserialized
                    .inner
                    .bind(py)
                    .repr()
                    .unwrap()
                    .to_str()
                    .unwrap(),
                foo.inner.bind(py).repr().unwrap().to_str().unwrap()
            );
        });
    }
}
