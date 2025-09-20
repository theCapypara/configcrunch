use std::collections::HashMap;
use std::mem::take;

use pyo3::IntoPyObjectExt;
pub(crate) use pyo3::exceptions;
pub(crate) use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple, PyType};

use crate::conv::{PyYamlConfigDocument, YcdDict, YcdValueType};
use crate::pyutil::ClonePyRef;
use crate::variables::{process_variables, process_variables_for};
use crate::{
    CircularDependencyError, InvalidDocumentError, InvalidHeaderError, REF, SchemaError,
    construct_new_ycd, delete_remove_markers, load_subdocuments, load_yaml_file,
    recursive_docs_to_dicts, resolve_and_merge,
};

/// A document represented by a dictionary, that can be validated,
/// can contain references to other (sub-)documents, which can be resolved,
/// and variables that can be parsed.
#[pyclass(module = "_main", subclass)]
#[derive(Debug)]
pub(crate) struct YamlConfigDocument {
    pub(crate) doc: YcdDict,
    /// The frozen Python representation of doc
    pub(crate) frozen: Option<Py<PyAny>>,
    #[pyo3(get, set)]
    pub(crate) path: Option<String>,
    #[pyo3(get, set)]
    pub(crate) parent_doc: Option<Py<YamlConfigDocument>>,
    #[pyo3(get, set)]
    pub(crate) absolute_paths: Vec<String>,
    pub(crate) bound_helpers: HashMap<String, Py<PyAny>>,
    pub(crate) already_loaded_docs: Option<Vec<String>>,
}

#[pymethods]
impl YamlConfigDocument {
    /// Constructs a YamlConfigDocument
    ///
    /// :param document:       The document as a dict, without the header.
    /// :param path:           Path of the document absolute to the configured repositories.
    ///                        If this is not from a repo, leave at None.
    /// :param parent:         Parent document
    #[new]
    #[pyo3(signature=(
    document,
    path = None,
    parent_doc = None,
    already_loaded_docs = None,
    absolute_paths = None
    ))]
    pub(crate) fn new(
        document: YcdDict,
        path: Option<String>,
        parent_doc: Option<Py<YamlConfigDocument>>,
        already_loaded_docs: Option<Vec<String>>,
        absolute_paths: Option<Vec<String>>,
    ) -> PyResult<Self> {
        let already_loaded_docs = already_loaded_docs.unwrap_or_default();
        let absolute_paths = absolute_paths.unwrap_or_default();

        let mut slf = Self {
            doc: document,
            frozen: None,
            path,
            bound_helpers: HashMap::new(),
            absolute_paths,
            parent_doc,
            already_loaded_docs: None,
        };

        slf.infinite_recursion_check(already_loaded_docs)?;
        Ok(slf)
    }

    /// Constructs a YamlConfigDocument from a YAML-file.
    ///
    /// Expects the content to be a dictionary with one key (defined in the
    /// header method) and it's value is the body of the document,
    /// validated by the schema method.
    #[classmethod]
    pub(crate) fn from_yaml(
        cls: Py<PyType>,
        py: Python,
        path_to_yaml: String,
    ) -> PyResult<PyYamlConfigDocument> {
        let mut entire_document = load_yaml_file(&path_to_yaml)?;
        let header = cls.getattr(py, "header")?.call0(py)?;
        let header: &str = header.extract(py)?;
        if !entire_document.contains_key(header) {
            return Err(InvalidHeaderError::new_err(format!(
                "The document does not have a valid header. Expected was: {}",
                header
            )));
        }
        let content = entire_document.remove(header).unwrap();
        match content {
            YcdValueType::Dict(c) => construct_new_ycd(
                py,
                &cls,
                [
                    cls.clone_ref(py).into_any(),
                    c.into_py_any(py)?,
                    py.None(),
                    py.None(),
                    py.None(),
                    vec![path_to_yaml].into_py_any(py)?,
                ],
            ),
            _ => Err(InvalidDocumentError::new_err(format!(
                "The document at {} is invalid",
                path_to_yaml
            ))),
        }
    }

    #[classmethod]
    pub(crate) fn from_dict(
        cls: Bound<PyType>,
        py: Python,
        dict: Py<PyAny>,
    ) -> PyResult<PyYamlConfigDocument> {
        construct_new_ycd(
            py,
            &cls.as_unbound().clone_ref(py),
            [
                cls.into_py_any(py)?,
                dict,
                py.None(),
                py.None(),
                py.None(),
                py.None(),
            ],
        )
    }

    /// Header that YAML-documents must contain.
    #[classmethod]
    pub(crate) fn header(_cls: Bound<PyType>) -> PyResult<String> {
        debug_assert!(
            false,
            "The class method header must be implemented. Do not call the parent method."
        );
        Err(exceptions::PyTypeError::new_err(
            "The class method header must be implemented. Do not call the parent method.",
        ))
    }

    /// Schema that the document should be validated against.
    #[classmethod]
    pub(crate) fn schema(_cls: Bound<PyType>) -> PyResult<Py<PyAny>> {
        debug_assert!(
            false,
            "The class method schema must be implemented. Do not call the parent method."
        );
        Err(exceptions::PyTypeError::new_err(
            "The class method schema must be implemented. Do not call the parent method.",
        ))
    }

    /// Specifies the subdocuments.
    ///
    /// A list of tuples, where:
    /// - The first element is the path to the element, with part pieces (nested dicts) seperated by "/".
    ///   If the path ends with [] and at that location is either a list or a dict, then all values will be converted.
    ///   Otherwise only the exact specified path will be converted, it must be a dict, matching the schema.
    /// - The second element is the referenced document type
    ///
    /// Example for tuples for a given dict::
    ///
    ///     dict = {"a": {"b": ... }, "c": [ ..., ... ], "d": {"1": ..., "2": ...}}
    ///     single = ("a/b": ...)
    ///     on_list = ("a/c[]": ...)
    ///     on_dict = ("a/d[]": ...)
    #[classmethod]
    fn subdocuments(_cls: Bound<PyType>) -> PyResult<Py<PyAny>> {
        debug_assert!(
            false,
            "The class method subdocuments must be implemented. Do not call the parent method."
        );
        Err(exceptions::PyTypeError::new_err(
            "The class method subdocuments must be implemented. Do not call the parent method.",
        ))
    }

    /// Validates the document against the Schema.
    pub(crate) fn validate(slf: &Bound<Self>, py: Python) -> PyResult<bool> {
        if slf.borrow().frozen.is_some() {
            return Err(exceptions::PyRuntimeError::new_err(
                "Document is already frozen.",
            ));
        }
        let self_: PyRef<Self> = slf.borrow();
        let args = PyTuple::new(py, [(&self_.doc).into_py_any(py)?])?;
        slf.getattr("schema")?
            .call0()?
            .getattr("validate")?
            .call1(args)?;
        Ok(true)
    }

    /// Resolve the $ref entry at the beginning of the document body and merge with referenced documents
    /// (changes this document in place).
    ///
    /// :param lookup_paths: Paths to the repositories, where referenced should be looked up.
    ///
    ///  :final: Since 0.2.0 this function must not be extended. Starting with 1.0.0, subclasses
    ///          overriding this method will be ignored.
    ///
    ///  :returns: self
    pub(crate) fn resolve_and_merge_references(
        slf: Py<Self>,
        py: Python,
        lookup_paths: Vec<String>,
    ) -> PyResult<Py<YamlConfigDocument>> {
        if slf.borrow(py).frozen.is_some() {
            return Err(exceptions::PyRuntimeError::new_err(
                "Document is already frozen.",
            ));
        }
        let slf_clone = slf.clone_ref(py);

        if let Ok(cb) = slf.getattr(py, "_initialize_data_before_merge") {
            let mut mref = slf.borrow_mut(py);
            let args = PyTuple::new(py, [take(&mut mref.doc)])?;
            drop(mref);
            let tmp = cb.call1(py, args)?.extract(py)?;
            let mut mref = slf.borrow_mut(py);
            mref.doc = tmp;
            drop(mref);
        }

        resolve_and_merge(py, slf.clone_ref(py).into(), &lookup_paths)?;

        if let Ok(cb) = slf.getattr(py, "_initialize_data_after_merge") {
            let mut mref = slf.borrow_mut(py);
            let args = PyTuple::new(py, [take(&mut mref.doc).into_py_any(py)?])?;
            drop(mref);
            let tmp = cb.call1(py, args)?.extract(py)?;
            let mut mref = slf.borrow_mut(py);
            mref.doc = tmp;
            drop(mref);
        }

        let subdoc_spec = slf.call_method0(py, "subdocuments")?.extract(py)?;
        load_subdocuments(py, slf.clone_ref(py).into(), subdoc_spec, &lookup_paths)?;

        let mut self_: PyRefMut<Self> = slf.borrow_mut(py);
        let d = take(&mut self_.doc);
        match delete_remove_markers(py, YcdValueType::Dict(d))? {
            YcdValueType::Dict(dd) => self_.doc = dd,
            _ => {
                return Err(exceptions::PyRuntimeError::new_err(
                    "Internal algorithm failure.",
                ));
            }
        }
        Ok(slf_clone)
    }

    /// Process all {{ variables }} inside this document and all sub-documents.
    ///  All references must be resolved beforehand to work correctly (resolve_and_merge_references).
    ///  Changes this document in place.
    fn process_vars(slf: Py<Self>, py: Python) -> PyResult<Py<Self>> {
        if slf.borrow(py).frozen.is_some() {
            return Err(exceptions::PyRuntimeError::new_err(
                "Document is already frozen.",
            ));
        }
        process_variables(py, slf.clone_ref(py).into())?;
        if let Ok(cb) = slf.getattr(py, "_initialize_data_after_variables") {
            let mut mref = slf.borrow_mut(py);
            let args = PyTuple::new(py, take(&mut mref.doc))?;
            drop(mref);
            let tmp = cb.call1(py, args)?.extract(py)?;
            let mut mref = slf.borrow_mut(py);
            mref.doc = tmp;
        }
        Ok(slf)
    }

    /// Process all {{ variables }} inside the specified string as if it were part of this document.
    //  All references must be resolved beforehand to work correctly (resolve_and_merge_references).
    //
    //  additional_helpers may contain additional variable helper functions to use.
    fn process_vars_for(
        slf: Py<Self>,
        py: Python,
        target: &str,
        additional_helpers: Vec<Py<PyAny>>,
    ) -> PyResult<YcdValueType> {
        process_variables_for(py, slf.into(), target, additional_helpers)
    }

    /// .. admonition:: Variable Helper
    ///
    ///     Can be used inside configuration files.
    ///
    /// A helper function that can be used by variable-placeholders to the get the parent document (if any is set).
    ///
    ///  Example usage::
    ///
    ///      something: '{{ parent().field }}'
    ///
    ///  Example result::
    ///
    ///      something: 'value of parent field'
    fn parent(slf: PyRef<Self>, py: Python) -> PyResult<Py<PyAny>> {
        match &slf.parent_doc {
            None => Ok(py.None()),
            Some(x) => x.into_py_any(py),
        }
    }

    /// Copies the internal data to make it accessible via self.doc and self[...].
    /// You can not call resolve_and_merge_references, process_vars or validate on a frozen document.
    /// If you (still) need to use these, consider using the 'internal_*' methods instead.
    fn freeze(slf: Py<YamlConfigDocument>, py: Python) -> PyResult<()> {
        recursive_ycd_do(
            slf.into(),
            |ycd| {
                let mut borrow = ycd.borrow_mut(py);
                borrow.frozen = Some((&borrow.doc).into_py_any(py)?);
                if let Ok(cb) = ycd.getattr(py, "_initialize_data_after_freeze") {
                    drop(borrow);
                    cb.call0(py).ok();
                };
                Ok(())
            },
            py,
        )
    }

    #[getter]
    /// Representation of the internal data. Object needs to be frozen first, otherwise this will raise a TypeError.
    fn doc(&self, py: Python) -> PyResult<Py<PyAny>> {
        match &self.frozen {
            None => {
                //debug_assert!(false, "Document needs to be frozen first.");
                Err(exceptions::PyAttributeError::new_err(
                    "Document needs to be frozen first.",
                ))
            }
            Some(v) => Ok(v.clone_ref(py)),
        }
    }

    /// Error string representation.
    /// This short string representation is used in Schema errors and is meant to assist in finding
    /// document errors. Set this to a small representation of the document, that the user can understand.
    fn error_str(slf: PyRef<Self>, py: Python) -> PyResult<String> {
        Ok(Self::error_str_internal(
            &slf.into_py_any(py)?
                .getattr(py, "__class__")?
                .getattr(py, "__name__")?
                .to_string(),
        ))
    }

    fn __repr__(slf: Py<Self>, py: Python) -> PyResult<String> {
        Self::__str__(slf, py)
    }

    pub(crate) fn __str__(slf: Py<Self>, py: Python) -> PyResult<String> {
        let typename = slf.getattr(py, "__class__")?.getattr(py, "__name__")?;
        match &slf.borrow(py).frozen {
            None => Ok(format!("{}(???)", typename)),
            Some(_) => {
                let dict = slf.getattr(py, "doc")?.getattr(py, "__str__")?.call0(py)?;
                Ok(format!("{}({})", typename, dict))
            }
        }
    }

    fn __len__(slf: Py<Self>, py: Python) -> PyResult<usize> {
        slf.getattr(py, "doc")?
            .getattr(py, "__len__")?
            .call0(py)?
            .extract(py)
    }

    fn __getitem__(slf: Py<Self>, py: Python, key: &str) -> PyResult<Py<PyAny>> {
        let args = PyTuple::new(py, [key])?;
        slf.getattr(py, "doc")?
            .getattr(py, "__getitem__")?
            .call1(py, args)
    }

    fn __setitem__(slf: Py<Self>, py: Python, key: String, value: YcdValueType) -> PyResult<()> {
        let args = PyTuple::new(py, [key.into_py_any(py)?, value.into_py_any(py)?])?;
        slf.getattr(py, "doc")?
            .getattr(py, "__setitem__")?
            .call1(py, args)?;
        Ok(())
    }

    fn __delitem__(slf: Py<Self>, key: &str, py: Python) -> PyResult<()> {
        let args = PyTuple::new(py, [key])?;
        slf.getattr(py, "doc")?
            .getattr(py, "__detitem__")?
            .call1(py, args)?;
        Ok(())
    }

    fn __iter__(slf: PyRef<Self>) -> PyResult<Py<PyAny>> {
        slf.doc(slf.py())?
            .getattr(slf.py(), "__iter__")?
            .call0(slf.py())
    }

    fn items(slf: Py<Self>, py: Python) -> PyResult<Py<PyAny>> {
        slf.getattr(py, "doc")
    }

    fn to_dict(slf: Py<Self>, py: Python) -> PyResult<Py<PyAny>> {
        let frozen = &slf.borrow(py).frozen;
        match frozen {
            None => {
                let self_: PyRef<Self> = slf.borrow(py);
                let mut dict: YcdDict = HashMap::new();
                dict.insert(
                    slf.getattr(py, "header")?.call0(py)?.extract(py)?,
                    YcdValueType::Dict(self_.doc.clone_pyref(py)),
                );
                Ok(recursive_docs_to_dicts(YcdValueType::Dict(dict), py)?.into_py_any(py)?)
            }
            Some(_) => {
                // We are doing this from Python code for better readability
                let args = PyTuple::new(py, [slf.clone_ref(py)])?;
                Ok(py
                    .import("configcrunch._util")?
                    .getattr("frozen_ycd_to_dict")?
                    .call1(args)?
                    .into_py_any(py)?)
            }
        }
    }

    /// If not frozen: Returns a COPY of the key at the specified location
    /// Otherwise returns it from the frozen `self.doc`, it may or may not be a copy.
    fn internal_get(slf: &Bound<Self>, key: &str) -> PyResult<Py<PyAny>> {
        match &slf.borrow().frozen {
            None => slf.borrow().doc.get(key).into_py_any(slf.py()),
            Some(f) => f
                .extract::<Bound<PyDict>>(slf.py())?
                .get_item(key)?
                .into_py_any(slf.py()),
        }
    }

    /// If not frozen: Sets the value at the specified location in the internal document.
    /// Otherwise sets it it in the frozen `self.doc`.
    fn internal_set(slf: &Bound<Self>, key: String, val: YcdValueType) -> PyResult<()> {
        match &slf.borrow().frozen {
            None => { /*Drop borrow*/ }
            Some(f) => {
                f.extract::<Bound<PyDict>>(slf.py())?
                    .set_item(key, val.into_py_any(slf.py())?)?;
                return Ok(());
            }
        }
        // None:
        slf.borrow_mut().doc.insert(key, val);
        Ok(())
    }

    /// If not frozen: Returns whether the internal document contains `key`.
    /// Otherwise returns whether the frozen `self.doc` contains `key`.
    fn internal_contains(slf: &Bound<Self>, key: &str) -> PyResult<bool> {
        Ok(match &slf.borrow().frozen {
            None => slf.borrow().doc.contains_key(key),
            Some(f) => f.extract::<Bound<PyDict>>(slf.py())?.contains(key)?,
        })
    }

    /// If not frozen: Deletes a value from the internal document at `key`.
    /// Otherwise deletes a value from `self.doc` at `key`.
    fn internal_delete(slf: &Bound<Self>, key: &str) -> PyResult<()> {
        match &slf.borrow().frozen {
            None => { /* Drop borrow*/ }
            Some(f) => {
                f.extract::<Bound<PyDict>>(slf.py())?.del_item(key).ok();
                return Ok(());
            }
        };
        // None:
        slf.borrow_mut().doc.remove(key);
        Ok(())
    }

    /// Freezes the document temporarily (as long as the context is active, and synchronizes all
    /// data back into the internal document afterwards. Document must not already be frozen.
    fn internal_access(slf: Py<Self>) -> InternalAccessContext {
        InternalAccessContext(slf.into())
    }
}

impl YamlConfigDocument {
    /// Infinite recursion check
    fn infinite_recursion_check(&mut self, mut already_loaded_docs: Vec<String>) -> PyResult<()> {
        if let Some(path) = &self.path {
            if already_loaded_docs.contains(path) {
                return Err(CircularDependencyError::new_err(format!(
                    "Infinite circular reference detected while trying to load {}",
                    path
                )));
            }
            already_loaded_docs.push(path.clone());
        }
        self.already_loaded_docs = Some(already_loaded_docs);
        Ok(())
    }

    /// Loads bound variable helper methods to this instance for use in variable processing.
    pub(crate) fn collect_bound_variable_helpers<'py>(
        slf: Bound<'py, Self>,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, Self>> {
        let inspect = py.import("inspect")?;
        let ismethod = inspect.getattr("ismethod")?;
        let args = PyTuple::new(py, [(&slf).into_py_any(py)?, ismethod.into_py_any(py)?])?;
        let members: Bound<PyList> = inspect.getattr("getmembers")?.call1(args)?.extract()?;
        for tpl in members.iter() {
            let tpl: Bound<PyTuple> = tpl.extract()?;
            let itm = tpl.get_item(1)?;
            let name: String = tpl.get_item(0)?.extract()?;
            if itm.hasattr("__is_variable_helper")? {
                slf.borrow_mut()
                    .bound_helpers
                    .insert(name, itm.into_py_any(py)?);
            }
        }
        slf.borrow_mut().bound_helpers.insert(
            "parent".to_string(),
            slf.getattr("parent")?.into_py_any(py)?,
        );
        Ok(slf)
    }

    #[inline]
    pub(crate) fn error_str_internal(class_name: &str) -> String {
        format!("type {}", class_name)
    }
}

#[pyclass(module = "_main")]
struct InternalAccessContext(PyYamlConfigDocument);

#[pymethods]
impl InternalAccessContext {
    fn __enter__(&mut self, py: Python) -> PyResult<()> {
        YamlConfigDocument::freeze(self.0.clone_ref(py).into(), py)
    }

    #[pyo3(signature = (_exc_type=None, _exc_value=None, _traceback=None))]
    fn __exit__(
        &mut self,
        py: Python,
        _exc_type: Option<Py<PyAny>>,
        _exc_value: Option<Py<PyAny>>,
        _traceback: Option<Py<PyAny>>,
    ) -> PyResult<()> {
        recursive_ycd_do(
            self.0.clone_ref(py),
            |ycd| {
                let mut borrow = ycd.borrow_mut(py);
                match &borrow.frozen {
                    None => {}
                    Some(f) => {
                        borrow.doc = f.extract(py)?;
                        borrow.frozen = None;
                    }
                }
                Ok(())
            },
            py,
        )
    }
}

#[pyclass(module = "_main")]
pub(crate) struct DocReference {
    #[pyo3(get)]
    referenced_type: Py<PyType>, // Type[YamlConfigDocument]
}

#[pymethods]
impl DocReference {
    #[new]
    pub(crate) fn new(referenced_type: Py<PyType>) -> Self {
        Self { referenced_type }
    }

    fn __repr__(&self, py: Python) -> PyResult<String> {
        self.__str__(py)
    }

    fn __str__(&self, py: Python) -> PyResult<String> {
        let typename = self
            .referenced_type
            .extract::<Bound<PyType>>(py)?
            .getattr("__name__")?;
        Ok(format!("DocReference<{:?}>", typename))
    }

    /// Validates. If the subdocument still contains $ref, it is not validated further,
    /// please call resolve_and_merge_references. Otherwise the sub-document is expected to match
    /// according to it's schema.
    pub(crate) fn validate(slf: Py<Self>, data: Bound<PyAny>, py: Python) -> PyResult<bool> {
        let self_: PyRef<Self> = slf.borrow(py);
        if data.is_instance_of::<PyDict>() {
            let data: Bound<PyDict> = data.extract()?;
            // If the reference still contains the $ref keyword, it is treated as an
            // unmerged reference and not validated further.
            if data.contains(REF)? {
                return Ok(true);
            }
            return Err(SchemaError::new_err(format!(
                "Expected an instance of {:?} while validating, got {:?}: {:?}",
                self_
                    .referenced_type
                    .extract::<Bound<PyType>>(py)?
                    .getattr("__name__")?,
                data.getattr("__class__")?.getattr("__name__")?,
                data
            )));
        }

        let self_type = self_.referenced_type.extract::<Bound<PyType>>(py)?;
        if data.is_instance(&self_type)? {
            let data_doc: Bound<YamlConfigDocument> = data.extract()?;
            if data_doc.borrow().doc.contains_key(REF) {
                return Ok(true);
            }
            return data_doc.getattr("validate")?.call0()?.extract();
        }
        Err(SchemaError::new_err(format!(
            "Expected an instance of {:?} while validating, got {:?}: {:?}",
            self_
                .referenced_type
                .extract::<Bound<PyType>>(py)?
                .getattr("__name__")?,
            data.getattr("__class__")?.getattr("__name__")?,
            data
        )))
    }
}

fn recursive_ycd_do<F>(ycd: PyYamlConfigDocument, cb: F, py: Python) -> PyResult<()>
where
    F: (Fn(PyYamlConfigDocument) -> PyResult<()>) + Copy,
{
    _recursive_ycd_do_impl(&YcdValueType::Ycd(ycd), cb, py)
}

fn _recursive_ycd_do_impl<F>(obj: &YcdValueType, cb: F, py: Python) -> PyResult<()>
where
    F: (Fn(PyYamlConfigDocument) -> PyResult<()>) + Copy,
{
    match obj {
        YcdValueType::Ycd(v) => {
            cb(v.clone_ref(py))?;
            v.borrow(py)
                .doc
                .values()
                .try_for_each(|vv| _recursive_ycd_do_impl(vv, cb, py))
        }
        YcdValueType::Dict(v) => v
            .values()
            .try_for_each(|vv| _recursive_ycd_do_impl(vv, cb, py)),
        YcdValueType::List(v) => v
            .iter()
            .try_for_each(|vv| _recursive_ycd_do_impl(vv, cb, py)),
        _ => Ok(()),
    }
}
