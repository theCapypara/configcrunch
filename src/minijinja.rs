use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use crate::conv::{PyYamlConfigDocument, SimpleYcdValueType, YHashMap, YcdValueType};
use crate::pyutil::ClonePyRef;
use crate::{FORCE_STRING, YamlConfigDocument};
use minijinja::value::{Object, Value, ValueKind};
use minijinja::{Environment, Error, ErrorKind, State};
use pyo3::IntoPyObjectExt;
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use serde::{Serialize, Serializer};

// https://github.com/rust-lang/rust/issues/70263
macro_rules! typed_closure {
    (($($bound:tt)*), $closure:expr) => { {
        fn _typed_closure_id<F>(f: F) -> F where F: $($bound)* { f }
        _typed_closure_id($closure)
    } }
}

type FuncFunc = dyn Fn(&State, &[Value]) -> Result<Value, Error> + Sync + Send + 'static;

pub(crate) struct TemplateRenderer<'env> {
    env: Environment<'env>,
    document: PyYamlConfigDocument,
    globals: HashMap<String, Box<FuncFunc>>,
}

impl<'env> TemplateRenderer<'env> {
    const STR_FILTER: &'static str = "str";
    const SUBSTR_START_FILTER: &'static str = "substr_start";
    const STARTSWITH_FILTER: &'static str = "startswith";
    const TPL_NAME: &'static str = "tpl";

    pub(crate) fn new(document: PyYamlConfigDocument) -> PyResult<Self> {
        let mut slf = Self {
            env: Environment::new(),
            document,
            globals: HashMap::new(),
        };

        slf.env.add_filter(Self::STR_FILTER, str_filter);
        slf.env
            .add_filter(Self::STARTSWITH_FILTER, startswith_filter);
        slf.env
            .add_filter(Self::SUBSTR_START_FILTER, substr_start_filter);

        Ok(slf)
    }

    pub(crate) fn render(
        mut self,
        py: Python<'env>,
        input: &'env str,
    ) -> Result<Option<String>, Error> {
        if !input.contains('{') {
            // Shortcut if it doesn't contain any variables or control structures
            return Ok(None);
        }
        self.env.add_template(Self::TPL_NAME, input)?;
        let result = self
            .env
            .get_template(Self::TPL_NAME)?
            .render(Self::build_context(self.document.clone_ref(py)))?;
        self.env.remove_template(Self::TPL_NAME);
        Ok(Some(result))
    }

    pub(crate) fn add_helpers(&mut self, py: Python, helpers: Vec<Py<PyAny>>) {
        self.globals.extend(helpers.into_iter().map(|f| {
            (
                f.getattr(py, "__name__").unwrap().extract(py).unwrap(),
                Self::create_helper_fn(f),
            )
        }));
    }

    #[inline]
    fn build_context(document: PyYamlConfigDocument) -> Value {
        Value::from_object(document)
    }

    pub fn create_helper_fn(pyf: Py<PyAny>) -> Box<FuncFunc> {
        Box::new(typed_closure!(
            (Fn(&State, &[Value]) -> Result<Value, Error> + Sync + Send + 'static),
            move |_state: &State, args: &[Value]| -> Result<Value, Error> {
                Python::attach(|py| {
                    let pyargs = PyTuple::new(py, args.iter().cloned().map(WValue))
                        .expect("Failed to construct Python tuple");

                    match pyf.call1(py, pyargs) {
                        Ok(v) => match v.extract::<YcdValueType>(py) {
                            Ok(ycdvalue) => Ok(ycdvalue.into()),
                            Err(e) => convert_pyerr(e),
                        },
                        Err(e) => convert_pyerr(e),
                    }
                })
            }
        ))
    }
}

impl Display for PyYamlConfigDocument {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Python::attach(
            |py| match YamlConfigDocument::__str__(self.0.clone_ref(py), py) {
                Ok(v) => write!(f, "{}", v),
                Err(_) => write!(f, "YCD<?? Error during Display ??>",),
            },
        )
    }
}

fn str_filter(_state: &State, value: String) -> Result<String, Error> {
    Ok(Value::from(format!("{}{}", FORCE_STRING, value)).to_string())
}

fn substr_start_filter(_state: &State, string: String, start: usize) -> Result<String, Error> {
    Ok(string[start..].to_string())
}

fn startswith_filter(_state: &State, string: String, start: String) -> Result<bool, Error> {
    Ok(string.starts_with(&start))
}

fn convert_pyerr<_T>(in_e: pyo3::PyErr) -> Result<_T, Error> {
    Err(Error::new(
        ErrorKind::InvalidOperation,
        format!("Error in a function: {:?}", in_e),
    ))
}

impl From<SimpleYcdValueType> for Value {
    fn from(in_v: SimpleYcdValueType) -> Self {
        match in_v {
            SimpleYcdValueType::Dict(v) => Value::from_serialize(&v),
            SimpleYcdValueType::List(v) => Value::from(v),
            SimpleYcdValueType::YString(v) => Value::from(v),
            SimpleYcdValueType::Bool(v) => Value::from(v),
            SimpleYcdValueType::Int(v) => Value::from(v),
            SimpleYcdValueType::Float(v) => Value::from(v),
        }
    }
}

impl From<YcdValueType> for Value {
    fn from(in_v: YcdValueType) -> Self {
        match in_v {
            YcdValueType::Dict(v) => Value::from_object(YHashMap(v)),
            YcdValueType::List(v) => v
                .into_iter()
                .map(|v| v.into())
                .collect::<Vec<Value>>()
                .into(),
            YcdValueType::YString(v) => Value::from(v),
            YcdValueType::Bool(v) => Value::from(v),
            YcdValueType::Int(v) => Value::from(v),
            YcdValueType::Float(v) => Value::from(v),
            YcdValueType::Ycd(v) => Value::from_object(v),
        }
    }
}

impl From<&YcdValueType> for Value {
    fn from(in_v: &YcdValueType) -> Self {
        match in_v {
            YcdValueType::Dict(v) => {
                // TODO: Not ideal
                Python::attach(|py| Value::from_object(YHashMap(v.clone_pyref(py))))
            }
            YcdValueType::List(v) => v.iter().map(|v| v.into()).collect::<Vec<Value>>().into(),
            YcdValueType::YString(v) => Value::from(v.clone()),
            YcdValueType::Bool(v) => Value::from(*v),
            YcdValueType::Int(v) => Value::from(*v),
            YcdValueType::Float(v) => Value::from(*v),
            YcdValueType::Ycd(v) => Python::attach(|py| Value::from_object(v.clone_ref(py))),
        }
    }
}

struct WValue(Value);

impl<'py> IntoPyObject<'py> for WValue {
    type Target = PyAny; // the Python type
    type Output = Bound<'py, Self::Target>; // in most cases this will be `Bound`
    type Error = pyo3::PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(match self.0.kind() {
            ValueKind::Undefined => py.None().into_bound(py),
            ValueKind::None => py.None().into_bound(py),
            ValueKind::Bool => self.0.is_true().into_bound_py_any(py)?,
            ValueKind::Number => i128::try_from(self.0.clone())
                .unwrap()
                .into_bound_py_any(py)?,
            ValueKind::String => self.0.as_str().unwrap().into_bound_py_any(py)?,
            ValueKind::Bytes => self.0.as_bytes().into_bound_py_any(py)?,
            ValueKind::Seq => py.None().into_bound(py), // not supported
            ValueKind::Map => py.None().into_bound(py), // not supported
            ValueKind::Iterable => py.None().into_bound(py), // not supported
            ValueKind::Plain => py.None().into_bound(py), // not supported
            ValueKind::Invalid => py.None().into_bound(py), // not supported
            _ => py.None().into_bound(py),              // not supported
        })
    }
}

#[derive(Debug)]
struct VariableHelper(Py<PyAny>);

impl Display for VariableHelper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Object for VariableHelper {
    fn call(self: &Arc<Self>, state: &State, args: &[Value]) -> Result<Value, Error> {
        Python::attach(|py| TemplateRenderer::create_helper_fn(self.0.clone_ref(py))(state, args))
    }
}

impl Object for PyYamlConfigDocument {
    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        let name = key.as_str()?;
        Python::attach(|py| {
            let mut bow = self.0.borrow(py);
            bow.doc.get(name).map(|x| x.into()).or_else(|| {
                if bow.bound_helpers.is_empty() {
                    drop(bow);
                    YamlConfigDocument::collect_bound_variable_helpers(
                        self.0.clone_ref(py).into_bound(py),
                        py,
                    )
                    .ok();
                    bow = self.0.borrow(py);
                }
                bow.bound_helpers
                    .get(name)
                    .map(|x| Value::from_object(VariableHelper(x.clone_ref(py))))
            })
        })
    }

    fn call_method(
        self: &Arc<Self>,
        state: &State,
        name: &str,
        args: &[Value],
    ) -> Result<Value, Error> {
        Python::attach(|py| {
            let mut bow = self.0.borrow(py);
            if bow.bound_helpers.is_empty() {
                drop(bow);
                YamlConfigDocument::collect_bound_variable_helpers(
                    self.0.clone_ref(py).into_bound(py),
                    py,
                )
                .map_err(|e| convert_pyerr::<bool>(e).unwrap_err())?;
                bow = self.0.borrow(py);
            }
            match bow.bound_helpers.get(name) {
                None => Err(Error::new(
                    ErrorKind::InvalidOperation,
                    format!("Method {} not found on object", name),
                )),
                Some(helper) => {
                    TemplateRenderer::create_helper_fn(helper.clone_ref(py))(state, args)
                }
            }
        })
    }
}

struct YHashMapItem<'a>(String, &'a YcdValueType);
impl Serialize for YHashMapItem<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_some(&(&self.0, self.1))
    }
}

impl Object for YHashMap<String, YcdValueType> {
    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        let name = key.as_str()?;
        self.0.get(name).map(|x| x.into())
    }

    fn call_method(
        self: &Arc<Self>,
        _state: &State,
        name: &str,
        _args: &[Value],
    ) -> Result<Value, Error> {
        match name {
            "items" => Ok(Value::from(
                self.0
                    .iter()
                    .map(|(k, v)| Value::from_serialize(YHashMapItem(k.clone(), v)))
                    .collect::<Vec<Value>>(),
            )),
            "values" => Python::attach(|py| {
                Ok(Value::from(
                    self.0
                        .values()
                        .map(|v| v.clone_pyref(py))
                        .collect::<Vec<YcdValueType>>(),
                ))
            }),
            "keys" => Ok(Value::from(self.0.keys().cloned().collect::<Vec<String>>())),
            _ => Err(Error::new(
                ErrorKind::InvalidOperation,
                format!("object has no method named {}", name),
            )),
        }
    }
}
