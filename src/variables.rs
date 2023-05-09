use crate::conv::YcdValueType::{Dict, Int, List, YString, Ycd};
use crate::conv::{PyYamlConfigDocument, YcdValueType};
use crate::minijinja::TemplateRenderer;
use crate::variables::DocumentTraverserCallbackType::{CurrentDoc, SubDoc};
use crate::{VariableProcessingError, FORCE_STRING};
use pyo3::{exceptions, PyAny, PyObject, PyResult, Python, ToPyObject};

struct DocumentTraverser;

enum DocumentTraverserCallbackType {
    SubDoc,
    CurrentDoc(PyYamlConfigDocument),
}

impl DocumentTraverser {
    pub(crate) fn run_subdoc_callback(py: Python, subdoc: &mut YcdValueType) -> PyResult<bool> {
        Self::traverse(py, &SubDoc, subdoc)
    }

    pub(crate) fn run_current_doc_callback(
        py: Python,
        subdoc: &mut YcdValueType,
        document: PyYamlConfigDocument,
    ) -> PyResult<bool> {
        Self::traverse(py, &CurrentDoc(document), subdoc)
    }

    fn traverse(
        py: Python,
        callback_type: &DocumentTraverserCallbackType,
        input_node: &mut YcdValueType,
    ) -> PyResult<bool> {
        match input_node {
            Dict(in_dict) => {
                let mut changed = false;
                for v in in_dict.values_mut() {
                    changed |= Self::traverse(py, callback_type, v)?;
                }
                Ok(changed)
            }
            List(in_list) => {
                let mut changed = false;
                for v in in_list.iter_mut() {
                    changed |= Self::traverse(py, callback_type, v)?;
                }
                Ok(changed)
            }
            _ => match callback_type {
                SubDoc => Self::process_variables_for_subdoc(py, input_node),
                CurrentDoc(base) => {
                    Self::process_variables_current_doc(py, input_node, base.clone_ref(py))
                }
            },
        }
    }

    fn process_variables_for_subdoc(py: Python, input_node: &mut YcdValueType) -> PyResult<bool> {
        match input_node {
            Ycd(in_ycd) => {
                process_variables(py, in_ycd.clone_ref(py))?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Recursive process variables
    /// The input node is changed in place immediately for dict entries and after processing
    /// the entire list for list entries.
    /// :return: Merge result of step.
    fn process_variables_current_doc(
        py: Python,
        input_node: &mut YcdValueType,
        document: PyYamlConfigDocument,
    ) -> PyResult<bool> {
        match input_node {
            YString(in_str) => {
                match apply_variable_resolution(
                    py,
                    in_str,
                    TemplateRenderer::new(document.clone_ref(py))?,
                ) {
                    Ok(opt_new_value) => {
                        if let Some(new_value) = opt_new_value {
                            let mut changed = false;
                            if let YString(snv) = &new_value {
                                changed = snv != in_str;
                            }
                            *input_node = new_value;
                            Ok(changed)
                        } else {
                            Ok(false)
                        }
                    }
                    Err(orig_err) => {
                        let err = VariableProcessingError::new_err(format!(
                            "Error processing a variable for document. Original value was {}. Document path: {}.",
                            in_str, document.borrow(py).absolute_paths[0]
                        ));
                        let err_obj: PyObject = err.to_object(py);
                        let err_pyany: &PyAny = err_obj.extract(py)?;
                        err_pyany.setattr("__cause__", orig_err.to_object(py))?;
                        Err(err)
                    }
                }
            }
            _ => Ok(false),
        }
    }
}

/// Process variables for a document in a single string
fn apply_variable_resolution<'env>(
    py: Python,
    input_str: &'env str,
    template_renderer: TemplateRenderer<'env>,
) -> PyResult<Option<YcdValueType>> {
    match template_renderer.render(py, input_str) {
        Ok(opt_result) => Ok(opt_result.map(|result| {
            if input_str != result {
                // Allow parsed ints to be read as such
                match result.strip_prefix(FORCE_STRING) {
                    None => match result.parse::<i64>().ok() {
                        None => YString(result),
                        Some(parsed) => Int(parsed),
                    },
                    Some(stripped) => YString(stripped.to_string()),
                }
            } else {
                YString(input_str.to_string())
            }
        })),
        Err(e) => Err(exceptions::PyValueError::new_err(format!(
            "Error processing a variable ({}): {:?}",
            input_str, e
        ))),
    }
}

/// Process all variables in a document
pub(crate) fn process_variables(py: Python, ycd: PyYamlConfigDocument) -> PyResult<()> {
    // TODO: The algorithm isn't very smart. It just runs over the
    //       document, replacing variables, until no replacements have been done.
    //       This should be improved in future versions.
    let mut doc = Dict(ycd.borrow(py).doc.clone());
    DocumentTraverser::run_subdoc_callback(py, &mut doc)?;
    doc = Dict(doc.unwrap_dict());
    loop {
        let changed = DocumentTraverser::run_current_doc_callback(py, &mut doc, ycd.clone_ref(py))?;
        ycd.borrow_mut(py).doc = doc.unwrap_dict();
        if !changed {
            break;
        }
        doc = Dict(ycd.borrow_mut(py).doc.clone());
    }
    Ok(())
}

#[inline]
pub(crate) fn process_variables_for(
    py: Python,
    ycd: PyYamlConfigDocument,
    target: &str,
    additional_helpers: Vec<PyObject>,
) -> PyResult<YcdValueType> {
    let mut renderer: TemplateRenderer = TemplateRenderer::new(ycd.clone_ref(py))?;
    renderer.add_helpers(py, additional_helpers);
    Ok(match apply_variable_resolution(py, target, renderer)? {
        None => YString(target.to_string()),
        Some(s) => s,
    })
}
