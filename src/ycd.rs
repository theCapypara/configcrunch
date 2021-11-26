use std::collections::HashMap;
pub use pyo3::exceptions;
pub use pyo3::prelude::*;
use pyo3::PyIterProtocol;
use pyo3::types::{PyDict, PyFunction, PyList, PyString, PyTuple, PyType};
use crate::{CircularDependencyError, construct_new_ycd, delete_remove_markers, InvalidDocumentError, InvalidHeaderError, load_yaml_file, recursive_docs_to_dicts, REF, resolve_and_merge, SchemaError};
use crate::conv::{PyYcdDict, YcdDict, YcdValueType};
use crate::conv::YcdValueType::{Dict, Ycd};
use crate::variables::{process_variables, process_variables_for};

/// A document represented by a dictionary, that can be validated,
///  can contain references to other (sub-)documents, which can be resolved,
///  and variables that can be parsed.
#[pyclass(module = "main", subclass)]
#[derive(Clone, Debug)]
pub struct YamlConfigDocument {
    #[pyo3(get, set)]
    pub doc: PyYcdDict,
    #[pyo3(get, set)]
    pub path: Option<String>,
    #[pyo3(get, set)]
    pub parent_doc: Option<Py<YamlConfigDocument>>,
    #[pyo3(get, set)]
    pub absolute_paths: Vec<String>,
    pub(crate) bound_helpers: HashMap<String, Py<PyFunction>>,
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
    pub fn new(
        py: Python, document: PyYcdDict, path: Option<String>,
        parent: Option<YamlConfigDocument>,
        already_loaded_docs: Option<Vec<String>>,
        absolute_paths: Option<Vec<String>>
    ) -> PyResult<Self> {
        let parent_doc = match parent {
            None => None,
            Some(x) =>  Some(Py::new(py, x)?)
        };
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
            path,
            bound_helpers: HashMap::new(),
            absolute_paths,
            parent_doc,
            already_loaded_docs: None
        };

        slf.infinite_recursion_check(already_loaded_docs)?;
        let slf_borrowed = PyCell::new(py, slf)?.borrow_mut();
        Self::collect_bound_variable_helpers(slf_borrowed, py)
    }

    /// Constructs a YamlConfigDocument from a YAML-file.
    ///
    /// Expects the content to be a dictionary with one key (defined in the
    /// header method) and it's value is the body of the document,
    /// validated by the schema method.
    #[classmethod]
    pub fn from_yaml(cls: &PyType, py: Python, path_to_yaml: String) -> PyResult<PyObject> {
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
    pub fn from_dict(cls: &PyType, py: Python) -> PyResult<Self> {
        todo!()
    }

    /// Header that YAML-documents must contain.
    pub fn header(&self) -> PyResult<String> {
        Err(exceptions::PyTypeError::new_err("The class method header must be implemented. Do not call the parent method."))
    }

    /// Schema that the document should be validated against.
    pub fn schema(&self) -> PyResult<PyObject> {
        Err(exceptions::PyTypeError::new_err("The class method schema must be implemented. Do not call the parent method."))
    }

    /// Validates the document against the Schema.
    pub fn validate(slf: Py<Self>, py: Python) -> PyResult<bool> {
        let self_: Self = slf.extract(py)?;
        let args = PyTuple::new(py, [self_.doc.to_object(py)]);
        slf.getattr(py, "schema")?.call0(py)?.getattr(py, "validate")?.call1(py, args)?;
        Ok(true)
    }

    /// May be used to initialize the document by adding / changing data.
    ///
    /// Called after resolve_and_merge_references.
    /// Use this for setting default values.
    fn _initialize_data_after_merge(&self) -> PyResult<()> {
        Ok(())
    }

    /// May be used to initialize the document by adding / changing data.
    ///
    /// Called after process_vars.
    /// Use this for setting internal values based on processed values in the document.
    fn _initialize_data_after_variables(&self) -> PyResult<()> {
        Ok(())
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
    pub fn resolve_and_merge_references(slf: Py<Self>, py: Python, lookup_paths: Vec<String>) -> PyResult<Py<YamlConfigDocument>> {
        resolve_and_merge(py, slf.clone_ref(py).into(), &lookup_paths)?;
        slf.getattr(py, "_initialize_data_after_merge")?.call0(py)?;
        let args = PyTuple::new(py, [lookup_paths]);
        slf.getattr(py, "_load_subdocuments")?.call1(py, args)?;
        delete_remove_markers(py, Ycd(slf.clone_ref(py).into()))?;
        Ok(slf)
    }

    /// Load sub-documents during the merging step.
    ///  Override this to load custom sub-documents.
    ///  Make sure to check if the value you are trying to load is $remove (constant REMOVE) first!
    fn _load_subdocuments(&mut self, _lookup_paths: Vec<String>) -> PyResult<()> {
        Ok(())
    }

    /// Process all {{ variables }} inside this document and all sub-documents.
    ///  All references must be resolved beforehand to work correctly (resolve_and_merge_references).
    ///  Changes this document in place.
    fn process_vars(slf: Py<Self>, py: Python) -> PyResult<Self> {
        process_variables(py, &mut slf.clone_ref(py).into())?;
        let self_ = slf.extract::<Self>(py)?;
        slf.getattr(py, "_initialize_data_after_variables")?.call0(py)?;
        Ok(self_)
    }

    /// Process all {{ variables }} inside the specified string as if it were part of this document.
    //  All references must be resolved beforehand to work correctly (resolve_and_merge_references).
    //
    //  additional_helpers may contain additional variable helper functions to use.
    fn process_vars_for(slf: Py<YamlConfigDocument>, py: Python, target: & str, additional_helpers: Vec<PyObject>) -> PyResult<String> {
        process_variables_for(py, &slf.into(), target, additional_helpers)
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

    fn __str__(slf: Py<Self>, py: Python) -> PyResult<String> {
        let typename = slf.getattr(py, "__class__")?.getattr(py, "__name__")?;
        let dict = slf.getattr(py, "doc")?.getattr(py, "__str__")?.call0(py)?;
        Ok(format!(
            "{}({})", typename, dict
        ))
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
        let self_: Self = slf.extract(py)?;
        let mut dict: YcdDict = HashMap::new();
        dict.insert(slf.getattr(py, "header")?.call0(py)?.extract(py)?, Dict(self_.doc));
        recursive_docs_to_dicts(Dict(dict.into()), py)
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
    fn collect_bound_variable_helpers(slf: PyRefMut<Self>, py: Python) -> PyResult<Self> {
        let slfpyobj = slf.into_py(py);
        let inspect = py.import("inspect")?;
        let ismethod = inspect.getattr("ismethod")?;
        let args = PyTuple::new(py, [
            slfpyobj.to_object(py), ismethod.to_object(py)
        ]);
        let mut slf: Self = slfpyobj.extract(py)?;
        let members: &PyList = inspect.getattr("getmembers")?.call1(args)?.extract()?;
        for tpl in members.iter() {
            let tpl: &PyTuple = tpl.extract()?;
            let itm = tpl.get_item(1)?;
            let name: String = tpl.get_item(0)?.extract()?;
            if itm.hasattr("__is_variable_helper")? || name == "parent" {
                slf.bound_helpers.insert(name, itm.extract()?);
            }
        }
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
        let self_: &Self = &*slf;
        self_.doc.getattr(slf.py(), "__iter__")?.call0(slf.py())
    }
}

#[pyclass(module = "main")]
#[derive(Clone)]
pub struct DocReference {
    referenced_type: Py<PyType> // Type[YamlConfigDocument]
}

#[pymethods]
impl DocReference {
    #[new]
    pub fn new(referenced_type: Py<PyType>) -> Self {
        Self { referenced_type }
    }

    /// Validates. If the subdocument still contains $ref, it is not validated further,
    /// please call resolve_and_merge_references. Otherwise the sub-document is expected to match
    /// according to it's schema.
    pub fn validate(slf: Py<Self>, data: &PyAny, py: Python) -> PyResult<bool> {
        let self_: Self = slf.extract(py)?;
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
            let data_doc: PyRef<YamlConfigDocument> = data.extract()?;
            if data_doc.doc.extract(py)?.contains_key(REF) {
                return Ok(true);
            }
            return match data.getattr("validate")?.call0() {
                Ok(x) => Ok(x.extract()?),
                Err(e) => {
                    if e.is_instance::<SchemaError>(py) {
                        let args = PyTuple::new(py, [
                            Py::from(PyString::new(py, format!("Error parsing subdocument {}.",
                                                               YamlConfigDocument::error_str(data_doc, py)?
                            ).as_str())),
                            e.into_instance(py).getattr(py, "errors")?
                        ]);
                        return Err(SchemaError::new_err::<PyObject>(args.into_py(py)));
                    }
                    Err(e)
                }
            }
        }
        Err(SchemaError::new_err(format!(
            "Expected an instance of {:?} while validating, got {:?}: {:?}",
            self_.referenced_type.extract::<&PyType>(py)?.getattr("__name__")?,
            data.getattr("__class__")?.getattr("__name__")?,
            data
        )))
    }
}
