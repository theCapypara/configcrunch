use crate::YamlConfigDocument;
use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyFloat, PyInt, PyList, PyString};
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};

pub(crate) type YcdDict = HashMap<String, YcdValueType>;
pub(crate) type YcdList = Vec<YcdValueType>;
#[derive(Debug)]
pub(crate) struct YHashMap<K, V>(pub(crate) HashMap<K, V>);

impl<K: Debug, V: Debug> Display for YHashMap<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PyYamlConfigDocument(pub(crate) Py<YamlConfigDocument>);

impl PyYamlConfigDocument {
    pub(crate) fn clone_ref(&self, py: Python) -> PyYamlConfigDocument {
        self.0.clone_ref(py).into()
    }
    pub(crate) fn getattr(&self, py: Python, attr: &str) -> PyResult<PyObject> {
        self.0.getattr(py, attr)
    }
    pub(crate) fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
    pub(crate) fn borrow<'py>(&'py self, py: Python<'py>) -> PyRef<'py, YamlConfigDocument> {
        self.0.borrow(py)
    }
    pub(crate) fn borrow_mut<'py>(&'py self, py: Python<'py>) -> PyRefMut<'py, YamlConfigDocument> {
        self.0.borrow_mut(py)
    }
}

impl IntoPy<PyObject> for PyYamlConfigDocument {
    fn into_py(self, py: Python) -> PyObject {
        self.0.into_py(py)
    }
}

#[derive(Serialize, Clone, Debug)]
#[serde(untagged)]
pub(crate) enum YcdValueType {
    Ycd(PyYamlConfigDocument),
    Dict(YcdDict),
    List(YcdList),
    YString(String),
    Bool(bool),
    Int(i64),
    Float(f64),
}

impl YcdValueType {
    pub(crate) fn unwrap_dict(self) -> YcdDict {
        if let YcdValueType::Dict(d) = self {
            d
        } else {
            panic!("Did not contain a dict.")
        }
    }
}

/// Same as YcdValueType but without any containing Ycd; for deserialization
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub(crate) enum SimpleYcdValueType {
    Dict(HashMap<String, SimpleYcdValueType>),
    List(Vec<SimpleYcdValueType>),
    YString(String),
    Bool(bool),
    Int(i64),
    Float(f64),
    //CatchAll(Py<PyAny>), // This extraction never fails
}

impl Display for YcdValueType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            YcdValueType::Ycd(_) => write!(f, "<a document>"),
            YcdValueType::Dict(_) => write!(f, "<a dictionary>"),
            YcdValueType::List(_) => write!(f, "<a list>"),
            YcdValueType::YString(v) => write!(f, "{}", v),
            YcdValueType::Bool(v) => write!(f, "{}", v),
            YcdValueType::Int(v) => write!(f, "{}", v),
            YcdValueType::Float(v) => write!(f, "{}", v),
        }
    }
}

impl Display for SimpleYcdValueType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SimpleYcdValueType::Dict(_) => write!(f, "<a dictionary>"),
            SimpleYcdValueType::List(_) => write!(f, "<a list>"),
            SimpleYcdValueType::YString(v) => write!(f, "{}", v),
            SimpleYcdValueType::Bool(v) => write!(f, "{}", v),
            SimpleYcdValueType::Int(v) => write!(f, "{}", v),
            SimpleYcdValueType::Float(v) => write!(f, "{}", v),
        }
    }
}

impl<'source> FromPyObject<'source> for YcdValueType {
    fn extract(v: &'source PyAny) -> PyResult<Self> {
        match v.get_type().name()? {
            "dict" => {
                if let Ok(v) = <HashMap<String, YcdValueType>>::extract(v) {
                    return Ok(YcdValueType::Dict(v));
                }
            }
            "list" => {
                if let Ok(v) = <Vec<YcdValueType>>::extract(v) {
                    return Ok(YcdValueType::List(v));
                }
            }
            "str" => {
                if let Ok(v) = <String>::extract(v) {
                    return Ok(YcdValueType::YString(v));
                }
            }
            "int" => {
                if let Ok(v) = <i64>::extract(v) {
                    return Ok(YcdValueType::Int(v));
                }
            }
            "bool" => {
                if let Ok(v) = <bool>::extract(v) {
                    return Ok(YcdValueType::Bool(v));
                }
            }
            "float" => {
                if let Ok(v) = <f64>::extract(v) {
                    return Ok(YcdValueType::Float(v));
                }
            }
            &_ => { /* Go to fallback*/ }
        }
        // Fallback
        if let Ok(v) = v.extract::<Py<YamlConfigDocument>>() {
            Ok(YcdValueType::Ycd(v.into()))
        } else if let Ok(v) = <String>::extract(v) {
            Ok(YcdValueType::YString(v))
        } else if let Ok(v) = <i64>::extract(v) {
            Ok(YcdValueType::Int(v))
        } else if let Ok(v) = <f64>::extract(v) {
            Ok(YcdValueType::Float(v))
        } else if let Ok(v) = <bool>::extract(v) {
            Ok(YcdValueType::Bool(v))
        } else if let Ok(v) = <Vec<YcdValueType>>::extract(v) {
            Ok(YcdValueType::List(v))
        } else if let Ok(v) = <HashMap<String, YcdValueType>>::extract(v) {
            Ok(YcdValueType::Dict(v))
        } else {
            Err(exceptions::PyTypeError::new_err(format!(
                "Could not map type for {:?}",
                v
            )))
        }
    }
}

impl IntoPy<PyObject> for YcdValueType {
    fn into_py(self, py: Python) -> PyObject {
        match self {
            YcdValueType::Ycd(v) => v.0.into_py(py),
            YcdValueType::Dict(v) => v.into_py(py),
            YcdValueType::List(v) => v.into_py(py),
            YcdValueType::YString(v) => v.into_py(py),
            YcdValueType::Bool(v) => v.into_py(py),
            YcdValueType::Int(v) => v.into_py(py),
            YcdValueType::Float(v) => v.into_py(py),
            //YcdValueType::CatchAll(v) => v.into_py(py)
        }
    }
}

impl ToPyObject for YcdValueType {
    fn to_object(&self, py: Python) -> PyObject {
        match self {
            YcdValueType::Ycd(v) => v.0.to_object(py), // TODO: Probably not the fastest choice...
            YcdValueType::Dict(v) => v.to_object(py),
            YcdValueType::List(v) => v.to_object(py),
            YcdValueType::YString(v) => v.to_object(py),
            YcdValueType::Bool(v) => v.to_object(py),
            YcdValueType::Int(v) => v.to_object(py),
            YcdValueType::Float(v) => v.to_object(py),
            //YcdValueType::CatchAll(v) => v.to_object(py)
        }
    }
}

impl Serialize for PyYamlConfigDocument {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Python::with_gil(|py| match self.0.extract::<YamlConfigDocument>(py) {
            Ok(ycd) => serializer.collect_map(ycd.doc),
            Err(_) => panic!("Internal serialization failed."),
        })
    }
}

impl From<Py<YamlConfigDocument>> for PyYamlConfigDocument {
    fn from(v: Py<YamlConfigDocument>) -> Self {
        Self(v)
    }
}

impl From<PyYamlConfigDocument> for Py<YamlConfigDocument> {
    fn from(v: PyYamlConfigDocument) -> Self {
        v.0
    }
}

impl From<SimpleYcdValueType> for YcdValueType {
    fn from(e: SimpleYcdValueType) -> Self {
        match e {
            SimpleYcdValueType::Dict(v) => YcdValueType::Dict(
                v.into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect::<YcdDict>(),
            ),
            SimpleYcdValueType::List(v) => {
                YcdValueType::List(v.into_iter().map(|x| x.into()).collect::<YcdList>())
            }
            SimpleYcdValueType::YString(v) => YcdValueType::YString(v),
            SimpleYcdValueType::Bool(v) => YcdValueType::Bool(v),
            SimpleYcdValueType::Int(v) => YcdValueType::Int(v),
            SimpleYcdValueType::Float(v) => YcdValueType::Float(v),
        }
    }
}

impl From<YcdValueType> for SimpleYcdValueType {
    fn from(e: YcdValueType) -> Self {
        match e {
            YcdValueType::Dict(v) => {
                SimpleYcdValueType::Dict(v.into_iter().map(|(k, v)| (k, v.into())).collect())
            }
            YcdValueType::List(v) => {
                SimpleYcdValueType::List(v.into_iter().map(|x| x.into()).collect())
            }
            YcdValueType::YString(v) => SimpleYcdValueType::YString(v),
            YcdValueType::Bool(v) => SimpleYcdValueType::Bool(v),
            YcdValueType::Int(v) => SimpleYcdValueType::Int(v),
            YcdValueType::Float(v) => SimpleYcdValueType::Float(v),
            _ => {
                panic!("Invalid unexpected internal conversion.")
            } // This should never happen.
        }
    }
}

impl From<YHashMap<String, SimpleYcdValueType>> for HashMap<String, YcdValueType> {
    fn from(h: YHashMap<String, SimpleYcdValueType>) -> Self {
        h.0.into_iter().map(|(k, v)| (k, v.into())).collect()
    }
}

#[inline]
pub(crate) fn pyany_to_simple_ycd(v: &PyAny) -> SimpleYcdValueType {
    if let Ok(v) = v.extract::<&PyDict>() {
        SimpleYcdValueType::from(v)
    } else if let Ok(v) = v.extract::<&PyString>() {
        SimpleYcdValueType::from(v)
    } else if let Ok(v) = v.extract::<&PyInt>() {
        SimpleYcdValueType::from(v)
    } else if let Ok(v) = v.extract::<&PyFloat>() {
        SimpleYcdValueType::from(v)
    } else if let Ok(v) = v.extract::<&PyList>() {
        SimpleYcdValueType::from(v)
    } else if let Ok(v) = v.extract::<&PyBool>() {
        SimpleYcdValueType::from(v)
    } else {
        // TODO: Support more?
        SimpleYcdValueType::Bool(false)
    }
}

impl From<&PyDict> for SimpleYcdValueType {
    fn from(v: &PyDict) -> Self {
        SimpleYcdValueType::Dict(
            v.into_iter()
                .map(|(k, v)| (pyany_to_simple_ycd(k).to_string(), pyany_to_simple_ycd(v)))
                .collect(),
        )
    }
}

impl From<&PyString> for SimpleYcdValueType {
    fn from(v: &PyString) -> Self {
        SimpleYcdValueType::YString(v.extract().unwrap())
    }
}

impl From<&PyInt> for SimpleYcdValueType {
    fn from(v: &PyInt) -> Self {
        SimpleYcdValueType::Int(v.extract().unwrap())
    }
}

impl From<&PyFloat> for SimpleYcdValueType {
    fn from(v: &PyFloat) -> Self {
        SimpleYcdValueType::Float(v.extract().unwrap())
    }
}

impl From<&PyList> for SimpleYcdValueType {
    fn from(v: &PyList) -> Self {
        SimpleYcdValueType::List(v.into_iter().map(pyany_to_simple_ycd).collect())
    }
}

impl From<&PyBool> for SimpleYcdValueType {
    fn from(v: &PyBool) -> Self {
        SimpleYcdValueType::Bool(v.is_true())
    }
}
