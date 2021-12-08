use std::collections::HashMap;
use pyo3::{exceptions, PyAny, PyObject, PyResult, Python, ToPyObject};
use pyo3::types::{PyDict, PyTuple};
use tera::{Context, Tera, Value};
use crate::variables::DocumentTraverserCallback::{CurrentDoc, SubDoc};
use crate::{FORCE_STRING, VariableProcessingError, YamlConfigDocument};
use crate::conv::{YcdDict, YcdPyErr, YcdValueType, py_to_simple_ycd};
use crate::conv::YcdValueType::{Dict, List, Ycd, YString};

struct DocumentTraverser(bool); // something_changed

enum DocumentTraverserCallback<'a> {
    SubDoc,
    CurrentDoc(&'a YamlConfigDocument),
}

impl DocumentTraverser {
    pub fn run_subdoc_callback(py: Python, subdoc: YcdDict) -> PyResult<(YcdDict, bool)> {
        let mut slf = Self(false);
        match slf.traverse(py, SubDoc, Dict(subdoc))? {
            Dict(v) => Ok((v, slf.0)),
            _ => Err(exceptions::PyRuntimeError::new_err("Error in the document traversing algorithm."))
        }
    }

    pub fn run_current_doc_callback(py: Python, subdoc: YcdDict, document: &YamlConfigDocument) -> PyResult<(YcdDict, bool)>  {
        let mut slf = Self(false);
        match slf.traverse(py, CurrentDoc(document), Dict(subdoc))? {
            Dict(v) => Ok((v, slf.0)),
            _ => Err(exceptions::PyRuntimeError::new_err("Error in the document traversing algorithm."))
        }
    }

    fn traverse(&mut self, py: Python, callback: DocumentTraverserCallback, input_node: YcdValueType) -> PyResult<YcdValueType> {
        match input_node {
            Dict(in_dict) => {
                match in_dict.into_iter()
                    .map(|(k, v)| {
                        match { match callback {
                            SubDoc => self.process_variables_for_subdoc(py, v),
                            CurrentDoc(base) => self.process_variables_current_doc(py, v, base)
                        } } {
                            Ok(ov) => Ok((k, ov)),
                            Err(e) => Err(e)
                        }
                    }).collect::<PyResult<YcdDict>>() {
                        Ok(v) => Ok(Dict(v)),
                        Err(e) => Err(e)
                }
            }
            List(in_list) => {
                match in_list.into_iter().map(|x| match callback {
                    SubDoc => self.process_variables_for_subdoc(py, x),
                    CurrentDoc(base) => self.process_variables_current_doc(py, x, base)
                }).collect::<PyResult<Vec<YcdValueType>>>() {
                    Ok(v) => Ok(List(v)),
                    Err(e) => Err(e)
                }
            }
            _ => {
                match callback {
                    SubDoc => self.process_variables_for_subdoc(py, input_node),
                    CurrentDoc(base) => self.process_variables_current_doc(py, input_node, base)
                }
            }
        }
    }

    fn process_variables_for_subdoc(&self, py: Python, input_node: YcdValueType) -> PyResult<YcdValueType> {
        match input_node {
            Ycd(mut in_ycd) => {
                process_variables(py, &mut in_ycd.extract(py)?)?;
                Ok(Ycd(in_ycd))
            }
            _ => Ok(input_node)
        }
    }

    /// Recursive process variables
    /// The input node is changed in place immediately for dict entries and after processing
    /// the entire list for list entries.
    /// :return: Merge result of step.
    fn process_variables_current_doc(&mut self, py: Python, input_node: YcdValueType, document: &YamlConfigDocument) -> PyResult<YcdValueType> {
        match input_node {
            YString(in_str) => {
                match apply_variable_resolution(py, &in_str, document, vec![]) {
                    Ok(new_value) => {
                        if new_value != in_str {
                            self.0 = true;
                        }
                        Ok(YString(new_value))
                    }
                    Err(orig_err) => {
                        let err = VariableProcessingError::new_err(format!(
                            "Error processing a variable for document. Original value was {}. Document path: {}.",
                            in_str, document.absolute_paths[0]
                        ));
                        let err_obj: PyObject = err.to_object(py);
                        let err_pyany: &PyAny = err_obj.extract(py)?;
                        err_pyany.setattr("__cause__", orig_err.to_object(py))?;
                        Err(err)
                    }
                }
            }
            _ => Ok(input_node)
        }
    }
}

fn args_fill_dict(inp: &HashMap<String, Value>, out: &PyDict) -> PyResult<()> {
    for (k, v) in inp {
        let vv: YcdValueType = v.into();
        out.set_item(k, vv)?
    }
    Ok(())
}

// https://github.com/rust-lang/rust/issues/70263
macro_rules! typed_closure {
    (($($bound:tt)*), $closure:expr) => { {
        fn _typed_closure_id<F>(f: F) -> F where F: $($bound)* { f }
        _typed_closure_id($closure)
    } }
}

fn create_helper_fn(pyf: PyObject) -> Box<impl Fn(&HashMap<String, Value>) -> tera::Result<Value>> {
     Box::new(typed_closure!((Fn(&HashMap<String, Value>) -> tera::Result<Value>), move |args| -> tera::Result<Value> {
         Python::with_gil(|py| {
             let pyargs = PyTuple::empty(py);
             let pykwargs = PyDict::new(py);
             if let Err(e) = args_fill_dict(args, pykwargs) {
                 return Err(tera::Error::from(YcdPyErr(e)));
             }

             match pyf.call(py, pyargs, Some(pykwargs)) {
                 Ok(v) => {
                     Ok(py_to_simple_ycd(py, v).into())
                 },
                 Err(e) => Err(tera::Error::from(YcdPyErr(e)))
             }
         })
    }))
}

fn str_filter(x: &Value, _: &HashMap<String, Value>) -> Result<Value, tera::Error> {
    Ok(Value::String(format!("{}{}", FORCE_STRING, x)))
}

/// Process variables for a document in a single string
fn apply_variable_resolution(
    py: Python, input_str: &str, document: &YamlConfigDocument, additional_helpers: Vec<PyObject>
) -> PyResult<String> {
    let mut tera = Tera::default();

    // With inspiration from https://stackoverflow.com/a/47291097
    for (name, helper) in &document.bound_helpers {
        tera.register_function(name, create_helper_fn(helper.to_object(py)));
    }
    for helper in &additional_helpers {
        tera.register_function(helper.getattr(py, "__name__")?.extract(py)?, create_helper_fn(helper.clone_ref(py)));
    }
    tera.register_filter("str", str_filter);

    let context: Context;
    match Context::from_serialize(&document.doc) {
        Ok(c) => context = c,
        Err(e) => return Err(exceptions::PyRuntimeError::new_err(format!("Template initialization error: {:?}", e)))
    };
    let result: String;

    match tera.render("tpl", &context) {
        Ok(r) => result = r,
        Err(e) => return Err(exceptions::PyValueError::new_err(format!("Error processing a variable ({}): {:?}", input_str, e)))
    };

    // Allow parsed ints to be read as such
    if input_str != result {
        if !result.starts_with(FORCE_STRING) {
            return Ok(match result.parse::<i32>().ok() {
                None => result,
                Some(parsed) => parsed.to_string()
            });
        }
        return Ok(result[15..].to_string());
    }

    Ok(input_str.to_string())
}

/// Process all variables in a document
pub fn process_variables(py: Python, ycd: &mut YamlConfigDocument) -> PyResult<()> {
    // TODO: The algorithm isn't very smart. It just runs over the
    //       document, replacing variables, until no replacements have been done.
    //       This should be improved in future versions.
    // TODO: We have to clone here, because we need the original context later when processing vars.
    let mut res = DocumentTraverser::run_subdoc_callback(py, ycd.doc.clone())?.0;
    loop {
        let (inner_res, changed) = DocumentTraverser::run_current_doc_callback(py, res, ycd)?;
        res = inner_res;
        if ! changed { break; }
    }
    ycd.doc = res;
    Ok(())
}

#[inline]
pub fn process_variables_for(py: Python, ycd: &YamlConfigDocument, target: &str, additional_helpers: Vec<PyObject>) -> PyResult<String> {
    apply_variable_resolution(py, target, ycd, additional_helpers)
}
