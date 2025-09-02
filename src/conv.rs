use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};

use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyFloat, PyInt, PyList, PyString};
use serde::{Deserialize, Serialize, Serializer};

use crate::YamlConfigDocument;
use crate::pyutil::ClonePyRef;

pub(crate) type YcdDict = HashMap<String, YcdValueType>;
pub(crate) type YcdList = Vec<YcdValueType>;
#[derive(Debug)]
pub(crate) struct YHashMap<K, V>(pub(crate) HashMap<K, V>);

impl<K: Debug, V: Debug> Display for YHashMap<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl ClonePyRef for YcdDict {
    fn clone_pyref(&self, py: Python) -> Self {
        self.iter()
            .map(|(k, v)| (k.clone(), v.clone_pyref(py)))
            .collect()
    }
}

impl ClonePyRef for YcdList {
    fn clone_pyref(&self, py: Python) -> Self {
        self.iter().map(|v| v.clone_pyref(py)).collect()
    }
}

#[derive(Debug, IntoPyObject, IntoPyObjectRef)]
pub(crate) struct PyYamlConfigDocument(pub(crate) Py<YamlConfigDocument>);

impl PyYamlConfigDocument {
    pub(crate) fn clone_ref(&self, py: Python) -> PyYamlConfigDocument {
        self.0.clone_ref(py).into()
    }
    pub(crate) fn getattr(&self, py: Python, attr: &str) -> PyResult<Py<PyAny>> {
        self.0.getattr(py, attr)
    }
    pub(crate) fn borrow<'py>(&'py self, py: Python<'py>) -> PyRef<'py, YamlConfigDocument> {
        self.0.borrow(py)
    }
    pub(crate) fn borrow_mut<'py>(&'py self, py: Python<'py>) -> PyRefMut<'py, YamlConfigDocument> {
        self.0.borrow_mut(py)
    }
}

#[derive(Serialize, Debug, IntoPyObject, IntoPyObjectRef)]
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

// only as a crutch! consider using ClonePyRef instead.
impl Clone for YcdValueType {
    fn clone(&self) -> Self {
        Python::attach(|py| self.clone_pyref(py))
    }
}

impl ClonePyRef for YcdValueType {
    fn clone_pyref(&self, py: Python) -> Self {
        match self {
            Self::Ycd(v) => Self::Ycd(v.clone_ref(py)),
            Self::Dict(v) => Self::Dict(v.clone_pyref(py)),
            Self::List(v) => Self::List(v.clone_pyref(py)),
            Self::YString(v) => Self::YString(v.clone()),
            Self::Bool(v) => Self::Bool(*v),
            Self::Int(v) => Self::Int(*v),
            Self::Float(v) => Self::Float(*v),
        }
    }
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

impl<'py> FromPyObject<'py> for YcdValueType {
    fn extract_bound(v: &Bound<'py, PyAny>) -> PyResult<Self> {
        match v.get_type().name()?.to_str()? {
            "dict" => {
                if let Ok(v) = <HashMap<String, YcdValueType>>::extract_bound(v) {
                    return Ok(YcdValueType::Dict(v));
                }
            }
            "list" => {
                if let Ok(v) = <Vec<YcdValueType>>::extract_bound(v) {
                    return Ok(YcdValueType::List(v));
                }
            }
            "str" => {
                if let Ok(v) = <String>::extract_bound(v) {
                    return Ok(YcdValueType::YString(v));
                }
            }
            "int" => {
                if let Ok(v) = <i64>::extract_bound(v) {
                    return Ok(YcdValueType::Int(v));
                }
            }
            "bool" => {
                if let Ok(v) = <bool>::extract_bound(v) {
                    return Ok(YcdValueType::Bool(v));
                }
            }
            "float" => {
                if let Ok(v) = <f64>::extract_bound(v) {
                    return Ok(YcdValueType::Float(v));
                }
            }
            &_ => { /* Go to fallback*/ }
        }
        // Fallback
        if let Ok(v) = v.extract::<Py<YamlConfigDocument>>() {
            Ok(YcdValueType::Ycd(v.into()))
        } else if let Ok(v) = <String>::extract_bound(v) {
            Ok(YcdValueType::YString(v))
        } else if let Ok(v) = <i64>::extract_bound(v) {
            Ok(YcdValueType::Int(v))
        } else if let Ok(v) = <f64>::extract_bound(v) {
            Ok(YcdValueType::Float(v))
        } else if let Ok(v) = <bool>::extract_bound(v) {
            Ok(YcdValueType::Bool(v))
        } else if let Ok(v) = <Vec<YcdValueType>>::extract_bound(v) {
            Ok(YcdValueType::List(v))
        } else if let Ok(v) = <HashMap<String, YcdValueType>>::extract_bound(v) {
            Ok(YcdValueType::Dict(v))
        } else {
            Err(exceptions::PyTypeError::new_err(format!(
                "Could not map type for {:?}",
                v
            )))
        }
    }
}

impl Serialize for PyYamlConfigDocument {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Python::attach(|py| serializer.collect_map(&self.0.borrow(py).doc))
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
pub(crate) fn pyany_to_simple_ycd(v: Bound<PyAny>) -> SimpleYcdValueType {
    if let Ok(v) = v.extract::<Bound<PyDict>>() {
        SimpleYcdValueType::from(v)
    } else if let Ok(v) = v.extract::<Bound<PyString>>() {
        SimpleYcdValueType::from(v)
    } else if let Ok(v) = v.extract::<Bound<PyInt>>() {
        SimpleYcdValueType::from(v)
    } else if let Ok(v) = v.extract::<Bound<PyFloat>>() {
        SimpleYcdValueType::from(v)
    } else if let Ok(v) = v.extract::<Bound<PyList>>() {
        SimpleYcdValueType::from(v)
    } else if let Ok(v) = v.extract::<Bound<PyBool>>() {
        SimpleYcdValueType::from(v)
    } else {
        // TODO: Support more?
        SimpleYcdValueType::Bool(false)
    }
}

impl From<Bound<'_, PyDict>> for SimpleYcdValueType {
    fn from(v: Bound<PyDict>) -> Self {
        SimpleYcdValueType::Dict(
            v.into_iter()
                .map(|(k, v)| (pyany_to_simple_ycd(k).to_string(), pyany_to_simple_ycd(v)))
                .collect(),
        )
    }
}

impl From<Bound<'_, PyString>> for SimpleYcdValueType {
    fn from(v: Bound<PyString>) -> Self {
        SimpleYcdValueType::YString(v.extract().unwrap())
    }
}

impl From<Bound<'_, PyInt>> for SimpleYcdValueType {
    fn from(v: Bound<PyInt>) -> Self {
        SimpleYcdValueType::Int(v.extract().unwrap())
    }
}

impl From<Bound<'_, PyFloat>> for SimpleYcdValueType {
    fn from(v: Bound<PyFloat>) -> Self {
        SimpleYcdValueType::Float(v.extract().unwrap())
    }
}

impl From<Bound<'_, PyList>> for SimpleYcdValueType {
    fn from(v: Bound<PyList>) -> Self {
        SimpleYcdValueType::List(v.into_iter().map(pyany_to_simple_ycd).collect())
    }
}

impl From<Bound<'_, PyBool>> for SimpleYcdValueType {
    fn from(v: Bound<PyBool>) -> Self {
        SimpleYcdValueType::Bool(v.is_true())
    }
}
