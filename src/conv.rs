use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyFloat, PyInt, PyList, PyString};
use serde::{Deserialize, Serialize, Serializer};
use tera::{Number, Value};
use crate::YamlConfigDocument;

pub type YcdDict = HashMap<String, YcdValueType>;
pub type YcdList = Vec<YcdValueType>;

pub(crate) struct YcdPyErr(pub PyErr);
pub(crate) struct YHashMap<K, V>(pub HashMap<K, V>);

#[derive(Clone, FromPyObject, Debug)]
pub struct PyYamlConfigDocument(pub Py<YamlConfigDocument>);

#[derive(Clone, Debug)]
pub struct PyYcdDict(pub PyObject);

#[derive(Clone, Debug)]
pub struct PyYcdList(pub PyObject);

impl PyYamlConfigDocument {
    pub fn extract(&self, py: Python) -> PyResult<YamlConfigDocument> {
        self.0.extract(py)
    }
    pub fn clone_ref(&self, py: Python) -> Self {
        Self(self.0.clone_ref(py))
    }
    pub fn getattr(&self, py: Python, attr: &str) -> PyResult<PyObject> {
        self.0.getattr(py, attr)
    }
    pub fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

impl PyYcdDict {
    pub fn extract(&self, py: Python) -> PyResult<YcdDict> {
        self.0.extract(py)
    }
    pub fn clone_ref(&self, py: Python) -> Self {
        Self(self.0.clone_ref(py))
    }
    pub fn getattr(&self, py: Python, attr: &str) -> PyResult<PyObject> {
        self.0.getattr(py, attr)
    }
    pub fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

impl PyYcdList {
    pub fn extract(&self, py: Python) -> PyResult<YcdList> {
        self.0.extract(py)
    }
    pub fn clone_ref(&self, py: Python) -> Self {
        Self(self.0.clone_ref(py))
    }
    pub fn getattr(&self, py: Python, attr: &str) -> PyResult<PyObject> {
        self.0.getattr(py, attr)
    }
    pub fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

impl Default for PyYcdDict {
    fn default() -> Self {
        Python::with_gil(|py| YcdDict::new().to_object(py).into())
    }
}

#[derive(Serialize, Clone, Debug)]
pub enum YcdValueType {
    Ycd(PyYamlConfigDocument),
    Dict(PyYcdDict),
    List(PyYcdList),
    YString(String),
    Bool(bool),
    Int(i64),
    Float(f64),
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
            YcdValueType::Float(v) => write!(f, "{}", v)
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
            SimpleYcdValueType::Float(v) => write!(f, "{}", v)
        }
    }
}

impl FromPyObject<'source> for YcdValueType {
    fn extract(v: &'source PyAny) -> PyResult<Self> {
        if let Ok(vv) = v.extract::<Py<YamlConfigDocument>>() {
            Ok(YcdValueType::Ycd(vv.into()))
        } else if let Ok(v) = <String>::extract(v) {
            Ok(YcdValueType::YString(v))
        } else if let Ok(v) = <i64>::extract(v) {
            Ok(YcdValueType::Int(v  ))
        } else if let Ok(v) = <f64>::extract(v) {
            Ok(YcdValueType::Float(v))
        } else if let Ok(v) = <bool>::extract(v) {
            Ok(YcdValueType::Bool(v))
        }  else if let Ok(v) = <Vec<YcdValueType>>::extract(v) {
            Ok(YcdValueType::List(v.into()))
        }else if let Ok(v) = <HashMap<String, YcdValueType>>::extract(v) {
            Ok(YcdValueType::Dict(v.into()))
        } else {
            Err(exceptions::PyTypeError::new_err(format!("Could not map type for {:?}", v)))
         }
    }
}

impl FromPyObject<'source> for PyYcdDict {
    fn extract(v: &'source PyAny) -> PyResult<Self> {
        if let Ok(vv) = v.extract::<YcdDict>() {
            Ok(vv.into_py(v.py()).into())
        } else {
            Err(exceptions::PyTypeError::new_err(format!("Could not map type for {:?}", v)))
         }
    }
}

impl FromPyObject<'source> for PyYcdList {
    fn extract(v: &'source PyAny) -> PyResult<Self> {
        if let Ok(vv) = v.extract::<YcdList>() {
            Ok(vv.into_py(v.py()).into())
        } else {
            Err(exceptions::PyTypeError::new_err(format!("Could not map type for {:?}", v)))
         }
    }
}

impl From<&Value> for YcdValueType {
    fn from(v: &Value) -> Self {
        match v {
            Value::Null => YcdValueType::Bool(false),  // TODO: Not ideal!
            Value::Bool(c) => YcdValueType::Bool(*c),
            Value::Number(c) => {
                if c.is_i64() {
                    YcdValueType::Int(c.as_i64().unwrap())
                } else {
                    YcdValueType::Float(c.as_f64().unwrap())
                }
            }
            Value::String(c) => YcdValueType::YString(c.clone()),
            Value::Array(c) => YcdValueType::List(c.iter().map(|x| x.into()).collect::<YcdList>().into()),
            Value::Object(c) => YcdValueType::Dict(
                c.iter().map(|(k,v)| (k.clone(), v.into())).collect::<YcdDict>().into()
            )
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

impl IntoPy<PyObject> for PyYcdDict {
    fn into_py(self, py: Python) -> PyObject {
        self.0.into_py(py)
    }
}

impl IntoPy<PyObject> for PyYcdList {
    fn into_py(self, py: Python) -> PyObject {
        self.0.into_py(py)
    }
}

impl ToPyObject for YcdValueType {
    fn to_object(&self, py: Python) -> PyObject {
        match self {
            YcdValueType::Ycd(v) => v.0.clone().into_py(py), // TODO: Probably not the fastest choice...
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

impl From<YcdPyErr> for tera::Error {
    fn from(e: YcdPyErr) -> Self {
        tera::Error::from(e.0.to_string())
    }
}

impl From<PyErr> for YcdPyErr {
    fn from(e: PyErr) -> Self {
        YcdPyErr(e)
    }
}

impl Serialize for PyYamlConfigDocument {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        Python::with_gil(|py|
            match self.0.extract::<YamlConfigDocument>(py) {
                Ok(ycd) => match ycd.doc.extract(py) {
                    Ok(ycddoc) => serializer.collect_map(ycddoc),
                    Err(_) => panic!("Internal serialization failed.")
                },
                Err(_) => panic!("Internal serialization failed.")
            }
        )
    }
}

impl Serialize for PyYcdDict {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        Python::with_gil(|py|
            match self.0.extract::<YcdDict>(py) {
                Ok(r) => serializer.collect_map(r),
                Err(_) => panic!("Internal serialization failed.")
            }
        )
    }
}

impl Serialize for PyYcdList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        Python::with_gil(|py|
            match self.0.extract::<YcdList>(py) {
                Ok(r) => serializer.collect_seq(r),
                Err(_) => panic!("Internal serialization failed.")
            }
        )
    }
}

impl From<Py<YamlConfigDocument>> for PyYamlConfigDocument {
    fn from(v: Py<YamlConfigDocument>) -> Self {
        Self(v)
    }
}

impl From<PyObject> for PyYcdDict {
    fn from(v: PyObject) -> Self {
        Self(v)
    }
}

impl From<PyObject> for PyYcdList {
    fn from(v: PyObject) -> Self {
        Self(v)
    }
}

impl From<YcdDict> for PyYcdDict {
    fn from(v: YcdDict) -> Self {
        Python::with_gil(|py| Self(v.into_py(py)))
    }
}

impl From<YcdList> for PyYcdList {
    fn from(v: YcdList) -> Self {
        Python::with_gil(|py| Self(v.into_py(py)))
    }
}

impl From<SimpleYcdValueType> for YcdValueType {
    fn from(e: SimpleYcdValueType) -> Self {
        match e {
            SimpleYcdValueType::Dict(v) => YcdValueType::Dict(v.into_iter().map(|(k, v)| (k, v.into())).collect::<YcdDict>().into()),
            SimpleYcdValueType::List(v) => YcdValueType::List(v.into_iter().map(|x| x.into()).collect::<YcdList>().into()),
            SimpleYcdValueType::YString(v) => YcdValueType::YString(v),
            SimpleYcdValueType::Bool(v) => YcdValueType::Bool(v),
            SimpleYcdValueType::Int(v) => YcdValueType::Int(v),
            SimpleYcdValueType::Float(v) => YcdValueType::Float(v)
        }
    }
}

impl From<YcdValueType> for SimpleYcdValueType {
    fn from(e: YcdValueType) -> Self {
        match e {
            YcdValueType::Dict(v) => Python::with_gil(|py| SimpleYcdValueType::Dict(v.extract(py).unwrap().into_iter().map(|(k, v)| (k, v.into())).collect())),
            YcdValueType::List(v) => Python::with_gil(|py| SimpleYcdValueType::List(v.extract(py).unwrap().into_iter().map(|x| x.into()).collect())),
            YcdValueType::YString(v) => SimpleYcdValueType::YString(v),
            YcdValueType::Bool(v) => SimpleYcdValueType::Bool(v),
            YcdValueType::Int(v) => SimpleYcdValueType::Int(v),
            YcdValueType::Float(v) => SimpleYcdValueType::Float(v),
            _ => {panic!("Invalid unexpected internal conversion.")}  // This should never happen.
        }
    }
}

impl From<YHashMap<String, SimpleYcdValueType>> for HashMap<String, YcdValueType> {
    fn from(h: YHashMap<String, SimpleYcdValueType>) -> Self {
        h.0.into_iter().map(|(k, v)| (k, v.into())).collect()
    }
}

#[inline]
pub(crate) fn py_to_simple_ycd(py: Python, v: PyObject) -> SimpleYcdValueType {
    if let Ok(v) = v.extract::<&PyDict>(py) {
        SimpleYcdValueType::from(v)
    } else if let Ok(v) = v.extract::<&PyString>(py) {
        SimpleYcdValueType::from(v)
    } else if let Ok(v) = v.extract::<&PyInt>(py) {
        SimpleYcdValueType::from(v)
    } else if let Ok(v) = v.extract::<&PyFloat>(py) {
        SimpleYcdValueType::from(v)
    } else if let Ok(v) = v.extract::<&PyList>(py) {
        SimpleYcdValueType::from(v)
    } else if let Ok(v) = v.extract::<&PyBool>(py) {
        SimpleYcdValueType::from(v)
    } else {
        // TODO: Support more?
        SimpleYcdValueType::Bool(false)
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
        SimpleYcdValueType::Dict(v.into_iter().map(|(k, v)| (pyany_to_simple_ycd(k).to_string(), pyany_to_simple_ycd(v))).collect())
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

impl From<SimpleYcdValueType> for Value {
    fn from(v: SimpleYcdValueType) -> Self {
        match v {
            SimpleYcdValueType::Dict(v) => Value::Object(v.into_iter().map(|(k, v)| (k, v.into())).collect()),
            SimpleYcdValueType::List(v) => Value::Array(v.into_iter().map(|x| x.into()).collect()),
            SimpleYcdValueType::YString(v) => Value::String(v),
            SimpleYcdValueType::Bool(v) => Value::Bool(v),
            SimpleYcdValueType::Int(v) => Value::Number(Number::from(v)),
            SimpleYcdValueType::Float(v) => Value::Number(Number::from_f64(v).unwrap())
        }
    }
}