use std::collections::HashMap;
use std::mem::take;
pub(crate) use pyo3::exceptions;
pub(crate) use pyo3::prelude::*;
use pyo3::PyIterProtocol;
use pyo3::types::{PyDict, PyList, PyTuple, PyType};
use crate::{CircularDependencyError, construct_new_ycd, delete_remove_markers, InvalidDocumentError, InvalidHeaderError, load_subdocuments, load_yaml_file, recursive_docs_to_dicts, REF, resolve_and_merge, SchemaError};
use crate::conv::{PyYamlConfigDocument, YcdDict, YcdValueType};
use crate::conv::YcdValueType::Dict;
use crate::variables::{process_variables, process_variables_for};

/// A document represented by a dictionary, that can be validated,
///  can contain references to other (sub-)documents, which can be resolved,
///  and variables that can be parsed.
#[pyclass(module = "main", subclass)]
#[derive(Clone, Debug)]
pub(crate) struct YamlConfigDocument {
    pub(crate) doc: YcdDict,
    /// The frozen Python representation of doc
    pub(crate) frozen: Option<PyObject>,
    #[pyo3(get, set)]
    pub(crate) path: Option<String>,
    #[pyo3(get, set)]
    pub(crate) parent_doc: Option<Py<YamlConfigDocument>>,
    #[pyo3(get, set)]
    pub(crate) absolute_paths: Vec<String>,
    pub(crate) bound_helpers: HashMap<String, PyObject>,
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
    #[args(path="None", parent="None", already_loaded_docs="None", absolute_paths="None")]
    pub(crate) fn new(
        document: YcdDict, path: Option<String>,
        parent_doc: Option<Py<YamlConfigDocument>>,
        already_loaded_docs: Option<Vec<String>>,
        absolute_paths: Option<Vec<String>>
    ) -> PyResult<Self> {
        let already_loaded_docs = match already_loaded_docs {
            None => vec![],
            Some(x) =>  x
        };
        let absolute_paths = match absolute_paths {
            None => vec![],
            Some(x) =>  x
        };

        let mut slf = Self {
            doc: document,
            frozen: None,
            path,
            bound_helpers: HashMap::new(),
            absolute_paths,
            parent_doc,
            already_loaded_docs: None
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
    pub(crate) fn from_yaml(cls: &PyType, py: Python, path_to_yaml: String) -> PyResult<PyYamlConfigDocument> {
        let mut entire_document = load_yaml_file(&path_to_yaml)?;
        let header = cls.getattr("header")?.call0()?;
        let header: &str = header.extract()?;
        if !entire_document.contains_key(header) {
            return Err(InvalidHeaderError::new_err(format!("The document does not have a valid header. Expected was: {}", header)));
        }
        let content = entire_document.remove(header).unwrap();
        match content {
            YcdValueType::Dict(c) => {
                construct_new_ycd(py, cls, [
                    cls.into_py(py) , c.into_py(py), py.None(), py.None(), py.None(), vec![path_to_yaml].into_py(py)
                ])
            },
            _ => Err(InvalidDocumentError::new_err(format!("The document at {} is invalid", path_to_yaml)))
        }
    }

    #[classmethod]
    pub(crate) fn from_dict(_cls: &PyType, _py: Python) -> PyResult<Self> {
        todo!()
    }

    /// Header that YAML-documents must contain.
    #[classmethod]
    pub(crate) fn header(_cls: &PyType) -> PyResult<String> {
        debug_assert!(false, "The class method header must be implemented. Do not call the parent method.");
        Err(exceptions::PyTypeError::new_err("The class method header must be implemented. Do not call the parent method."))
    }

    /// Schema that the document should be validated against.
    #[classmethod]
    pub(crate) fn schema(_cls: &PyType) -> PyResult<PyObject> {
        debug_assert!(false, "The class method schema must be implemented. Do not call the parent method.");
        Err(exceptions::PyTypeError::new_err("The class method schema must be implemented. Do not call the parent method."))
    }

    /// Specifies the subdocuments.
    /// A list of tuples, where:
    /// - The first element is the path to the element, with part pieces (nested dicts) seperated by "/".
    ///   If the path ends with [] and at that location is either a list or a dict, then all values will be converted.
    ///   Otherwise only the exact specified path will be converted, it must be a dict, matching the schema.
    /// - The second element is the referenced document type
    ///
    /// Example for tuples for a given dict::
    ///     dict = {"a": {"b": ... }, "c": [ ..., ... ], "d": {"1": ..., "2": ...}}
    ///     single = ("a/b": ...)
    ///     on_list = ("a/c[]": ...)
    ///     on_dict = ("a/d[]": ...)
    #[classmethod]
    fn subdocuments(_cls: &PyType) -> PyResult<PyObject> {
        debug_assert!(false, "The class method subdocuments must be implemented. Do not call the parent method.");
        Err(exceptions::PyTypeError::new_err("The class method subdocuments must be implemented. Do not call the parent method."))
    }

    /// Validates the document against the Schema.
    pub(crate) fn validate(slf: &PyCell<Self>, py: Python) -> PyResult<bool> {
        let self_: PyRef<Self> = slf.borrow();
        let args = PyTuple::new(py, [self_.doc.to_object(py)]);
        slf.getattr("schema")?.call0()?.getattr("validate")?.call1(args)?;
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
    pub(crate) fn resolve_and_merge_references(slf: Py<Self>, py: Python, lookup_paths: Vec<String>) -> PyResult<Py<YamlConfigDocument>> {
        let slf_clone = slf.clone_ref(py);

        if let Ok(cb) = slf.getattr(py, "_initialize_data_before_merge") {
            let mut mref = slf.borrow_mut(py);
            let args = PyTuple::new(py, [take(&mut mref.doc)]);
            mref.doc = cb.call1(py, args)?.extract(py)?;
            drop(mref);
        }

        resolve_and_merge(py, slf.clone_ref(py).into(), &lookup_paths)?;

        if let Ok(cb) = slf.getattr(py, "_initialize_data_after_merge") {
            let mut mref = slf.borrow_mut(py);
            let args = PyTuple::new(py, [take(&mut mref.doc).into_py(py)]);
            mref.doc = cb.call1(py, args)?.extract(py)?;
            drop(mref);
        }

        let subdoc_spec = slf.call_method0(py, "subdocuments")?.extract(py)?;
        load_subdocuments(py, slf.clone_ref(py).into(), subdoc_spec, &lookup_paths)?;

        let mut self_: PyRefMut<Self> = slf.borrow_mut(py);
        let d = take(&mut self_.doc);
        match delete_remove_markers(py, Dict(d))? {
            Dict(dd) => self_.doc = dd,
            _ => return Err(exceptions::PyRuntimeError::new_err("Internal algorithm failure."))
        }
        Ok(slf_clone)
    }

    /// Process all {{ variables }} inside this document and all sub-documents.
    ///  All references must be resolved beforehand to work correctly (resolve_and_merge_references).
    ///  Changes this document in place.
    fn process_vars(slf: Py<Self>, py: Python) -> PyResult<Py<Self>> {
        process_variables(py, slf.clone_ref(py).into())?;
        if let Ok(cb) = slf.getattr(py, "_initialize_data_after_variables") {
            let mut mref = slf.borrow_mut(py);
            let args = PyTuple::new(py, take(&mut mref.doc));
            mref.doc = cb.call1(py, args)?.extract(py)?;
        }
        Ok(slf)
    }

    /// Process all {{ variables }} inside the specified string as if it were part of this document.
    //  All references must be resolved beforehand to work correctly (resolve_and_merge_references).
    //
    //  additional_helpers may contain additional variable helper functions to use.
    fn process_vars_for(slf: Py<Self>, py: Python, target: &str, additional_helpers: Vec<PyObject>) -> PyResult<YcdValueType> {
        process_variables_for(py, slf.into(), target, additional_helpers)
    }

    /// A helper function that can be used by variable-placeholders to the get the parent document (if any is set).
    ///
    ///  Example usage::
    ///
    ///      something: '{{ parent().field }}'
    ///
    ///  Example result::
    ///
    ///      something: 'value of parent field'
    ///
    /// .. admonition:: Variable Helper
    ///
    ///     Can be used inside configuration files.
    fn parent(slf: PyRef<Self>, py: Python) -> PyResult<PyObject> {
        Ok(match &slf.parent_doc {
            None => slf.into_py(py),
            Some(x) => x.to_object(py)
        })
    }

    /// Copies the internal data to make it accessible via self.doc and self[...].
    /// Changes made are overwritten when other documents are merged into this or variables are processed
    /// after freeze() was called, additionally you will need to call freeze again to update
    /// self.doc and self.[...] then.
    fn freeze(&self) -> PyResult<()> {
        todo!()
        /*if let Ok(cb) = slf.getattr(py, "_initialize_data_after_freeze") {
            cb.call0(py, args)?;
        }*/
    }

    #[getter]
    /// Representation of the internal data. Object needs to be frozen first, otherwise this will raise a TypeError.
    fn doc(&self, py: Python) -> PyResult<PyObject> {
        match &self.frozen {
            None => Err(exceptions::PyAttributeError::new_err("Document needs to be frozen first.")),
            Some(v) => Ok(v.clone_ref(py))
        }
    }

    /// Error string representation.
    /// This short string representation is used in Schema errors and is meant to assist in finding
    /// document errors. Set this to a small representation of the document, that the user can understand.
    fn error_str(slf: PyRef<Self>, py: Python) -> PyResult<String> {
        Ok(
            Self::error_str_internal(&slf.into_py(py).getattr(py, "__class__")?.getattr(py, "__name__")?.to_string())
        )
    }

    fn __repr__(slf: Py<Self>, py: Python) -> PyResult<String> {
        Self::__str__(slf, py)
    }

    pub(crate) fn __str__(slf: Py<Self>, py: Python) -> PyResult<String> {
        let typename = slf.getattr(py, "__class__")?.getattr(py, "__name__")?;
        match &slf.borrow(py).frozen {
            None => Ok(format!(
                    "{}(???)", typename
                )),
            Some(_) => {
                let dict = slf.getattr(py, "doc")?.getattr(py, "__str__")?.call0(py)?;
                Ok(format!(
                    "{}({})", typename, dict
                ))
            }
        }
    }

    fn __len__(slf: Py<Self>, py: Python) -> PyResult<usize> {
        slf.getattr(py, "doc")?.getattr(py, "__len__")?.call0(py)?.extract(py)
    }

    fn __getitem__(slf: Py<Self>, py: Python, key: &str) -> PyResult<PyObject> {
        let args = PyTuple::new(py, [key]);
        slf.getattr(py, "doc")?.getattr(py, "__getitem__")?.call1(py, args)
    }

    fn __setitem__(slf: Py<Self>, py: Python, key: String, value: YcdValueType) -> PyResult<()> {
        let args = PyTuple::new(py, [key.to_object(py), value.to_object(py)]);
        slf.getattr(py, "doc")?.getattr(py, "__setitem__")?.call1(py, args)?;
        Ok(())
    }

    fn __delitem__(slf: Py<Self>, key: &str, py: Python) -> PyResult<()> {
        let args = PyTuple::new(py, [key]);
        slf.getattr(py, "doc")?.getattr(py, "__detitem__")?.call1(py, args)?;
        Ok(())
    }

    fn items(slf: Py<Self>, py: Python) -> PyResult<PyObject> {
        slf.getattr(py, "doc")
    }

    fn to_dict(slf: Py<Self>, py: Python) -> PyResult<YcdValueType> {
        let self_: PyRef<Self> = slf.borrow(py);
        let mut dict: YcdDict = HashMap::new();
        dict.insert(slf.getattr(py, "header")?.call0(py)?.extract(py)?, Dict(self_.doc.clone()));
        recursive_docs_to_dicts(Dict(dict), py)
    }
}

impl YamlConfigDocument {
    /// Infinite recursion check
    fn infinite_recursion_check(&mut self, mut already_loaded_docs: Vec<String>) -> PyResult<()> {
        if let Some(path) = &self.path {
            if already_loaded_docs.contains(path) {
                return Err(CircularDependencyError::new_err(format!("Infinite circular reference detected while trying to load {}", path)))
            }
            already_loaded_docs.push(path.clone());
        }
        self.already_loaded_docs = Some(already_loaded_docs);
        Ok(())
    }

    /// Loads bound variable helper methods to this instance for use in variable processing.
    pub(crate) fn collect_bound_variable_helpers<'py>(slf: &'py PyCell<Self>, py: Python<'py>) -> PyResult<&'py PyCell<Self>> {
        let inspect = py.import("inspect")?;
        let ismethod = inspect.getattr("ismethod")?;
        let args = PyTuple::new(py, [
            slf.to_object(py), ismethod.to_object(py)
        ]);
        let members: &PyList = inspect.getattr("getmembers")?.call1(args)?.extract()?;
        for tpl in members.iter() {
            let tpl: &PyTuple = tpl.extract()?;
            let itm = tpl.get_item(1)?;
            let name: String = tpl.get_item(0)?.extract()?;
            if itm.hasattr("__is_variable_helper")? {
                slf.borrow_mut().bound_helpers.insert(name, itm.into_py(py));
            }
        }
        slf.borrow_mut().bound_helpers.insert("parent".to_string(), slf.getattr("parent")?.to_object(py));
        Ok(slf)
    }

    #[inline]
    pub(crate) fn error_str_internal(class_name: &str) -> String {
        format!("type {}", class_name)
    }
}

#[pyproto]
impl PyIterProtocol for YamlConfigDocument {
    fn __iter__(slf: PyRef<Self>) -> PyResult<PyObject> {
        slf.doc(slf.py())?.getattr(slf.py(), "__iter__")?.call0(slf.py())
    }
}

#[pyclass(module = "main")]
#[derive(Clone)]
pub(crate) struct DocReference {
    referenced_type: Py<PyType> // Type[YamlConfigDocument]
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
        let typename = self.referenced_type.extract::<&PyType>(py)?.getattr("__name__")?;
        Ok(format!("DocReference<{:?}>", typename))
    }

    /// Validates. If the subdocument still contains $ref, it is not validated further,
    /// please call resolve_and_merge_references. Otherwise the sub-document is expected to match
    /// according to it's schema.
    pub(crate) fn validate(slf: Py<Self>, data: &PyAny, py: Python) -> PyResult<bool> {
        let self_: PyRef<Self> = slf.borrow(py);
        if data.is_instance::<PyDict>()? {
            let data: &PyDict = data.extract()?;
            // If the reference still contains the $ref keyword, it is treated as an
            // unmerged reference and not validated further.
            if data.contains(REF)? {
                return Ok(true);
            }
            return Err(SchemaError::new_err(format!(
                "Expected an instance of {:?} while validating, got {:?}: {:?}",
                self_.referenced_type.extract::<&PyType>(py)?.getattr("__name__")?,
                data.getattr("__class__")?.getattr("__name__")?,
                data
            )));
        }

        let self_type = self_.referenced_type.extract::<&PyType>(py)?;
        if self_type.is_instance(data)? {
            let data_doc: &PyCell<YamlConfigDocument> = data.extract()?;
            if data_doc.borrow().doc.contains_key(REF) {
                return Ok(true);
            }
            return YamlConfigDocument::validate(data_doc, py);
        }
        Err(SchemaError::new_err(format!(
            "Expected an instance of {:?} while validating, got {:?}: {:?}",
            self_.referenced_type.extract::<&PyType>(py)?.getattr("__name__")?,
            data.getattr("__class__")?.getattr("__name__")?,
            data
        )))
    }
}
