use crate::conv::YcdValueType::YString;
use crate::conv::{PyYamlConfigDocument, SimpleYcdValueType, YHashMap, YcdDict};
use crate::{merge_documents, InvalidDocumentError, InvalidHeaderError, YamlConfigDocument, REF};
use path_absolutize::Absolutize;
use pyo3::exceptions;
pub(crate) use pyo3::prelude::*;
use pyo3::types::{PyTuple, PyType};
use std::collections::HashMap;
use std::env::current_dir;
use std::fs::File;
use std::path::PathBuf;

#[pyfunction]
#[pyo3(signature = (doc_type, *args))]
/// Loads (one or) multiple YAML files (paths specified by *args) into the
/// given YamlConfigDocument model.
/// The documents are merged as if the rightmost document "$ref"'ed the document left to it, etc.
/// until all documents are merged.  However ``resolve_and_merge_references`` is not called on the base model;
/// an optimized internal merging is done instead.
pub(crate) fn load_multiple_yml(
    py: Python,
    doc_type: &PyType,
    args: &PyTuple,
) -> PyResult<PyYamlConfigDocument> {
    if args.is_empty() {
        return Err(exceptions::PyTypeError::new_err(
            "At least one document path must be passed.",
        ));
    }
    let args = args.iter().map(|x| x.extract::<String>());
    let mut doc: Option<PyYamlConfigDocument> = None;
    for rarg in args {
        match rarg {
            Ok(arg) => {
                let new_doc = YamlConfigDocument::from_yaml(doc_type, py, arg.clone())?;
                doc = Some(match doc {
                    None => new_doc,
                    Some(d) => merge_documents(py, new_doc, d)?,
                });
            }
            Err(e) => return Err(e),
        }
    }
    Ok(doc.unwrap())
}

/// Load the full absolute paths to the repositories (lookup paths) stored on disk.
pub(crate) fn load_repos(lookup_paths: &[String]) -> Vec<String> {
    lookup_paths.iter().map(|p| to_abs_path(p)).collect()
}

fn to_abs_path(str: &str) -> String {
    let ch = str.chars().next().unwrap();
    if ch == '/' || ch == '\\' {
        return current_dir()
            .unwrap()
            .join(str)
            .to_str()
            .unwrap()
            .to_string();
    }
    str.to_string()
}

/// Convert a $ref-Path into a full path absolute to the root of the repositories
///
/// :param base_path: Path of the file that contained the $ref or None if document was not part of the repositories
/// :param reference_path: Entry in $ref field.
/// :return: final path inside the repositories
pub(crate) fn path_in_repo(base_path: &Option<String>, reference_path: &str) -> String {
    match base_path {
        None => reference_path.to_string(),
        // TODO: This isn't truly cross platform but should be OK
        Some(p) => {
            let path: PathBuf = [
                "/",
                PathBuf::from(p).parent().unwrap().to_str().unwrap(),
                reference_path,
            ]
            .iter()
            .collect();
            let path: String = path.to_str().unwrap().to_string();
            #[cfg(target_family = "windows")]
            {
                path.replace("\\", "/");
            }
            path
        }
    }
}

/// Appends the paths inside repositories to the lookup_paths/repository paths, building a unique
/// absolute path on the disc that is only missing the file extension.
///
/// :param ref_path_in_repo: Path of resource absolute to repository root
/// :param lookup_paths: Paths to the repositories, as stored in the configuration documents
pub(crate) fn absolute_paths(
    ref_path_in_repo: &str,
    lookup_paths: &[String],
) -> PyResult<Vec<String>> {
    let ref_path_in_repo_cln = ref_path_in_repo
        .strip_prefix('/')
        .unwrap_or(ref_path_in_repo);
    load_repos(lookup_paths)
        .iter()
        .map(|absolute_repo_path| {
            // TODO: Is this safe?
            Ok(format!("{}/{}", absolute_repo_path, ref_path_in_repo_cln))
        })
        .collect::<PyResult<Vec<String>>>()
}

/// Load the actual dictionaries at path by checking if files ending in .yml/.yaml exist.
pub(crate) fn load_dicts(path: &str) -> PyResult<Vec<YcdDict>> {
    let mut doc_dicts: Vec<YcdDict> = Vec::with_capacity(2);
    if let Some(f) = load_dicts_try_single_path(PathBuf::from(format!("{}.yml", path)))? {
        doc_dicts.push(f);
    }
    if let Some(f) = load_dicts_try_single_path(PathBuf::from(format!("{}.yaml", path)))? {
        doc_dicts.push(f);
    }
    Ok(doc_dicts)
}

fn load_dicts_try_single_path(path: PathBuf) -> PyResult<Option<YcdDict>> {
    if let Ok(c) = path.absolutize_virtually("/") {
        if c.exists() {
            return Ok(Some(load_yaml_file(c.to_str().unwrap())?));
        }
    }
    Ok(None)
}

pub(crate) fn load_yaml_file(path_to_yaml: &str) -> PyResult<YcdDict> {
    let file;
    match File::open(path_to_yaml) {
        Ok(v) => file = v,
        Err(e) => {
            return Err(InvalidDocumentError::new_err(format!(
                "Unable to open YAML file {}: {:?}",
                path_to_yaml, e
            )))
        }
    };

    match serde_yaml::from_reader::<File, HashMap<String, SimpleYcdValueType>>(file) {
        Ok(v) => Ok(YHashMap(v).into()),
        Err(e) => Err(InvalidDocumentError::new_err(format!(
            "Unable to read YAML file {}: {:?}",
            path_to_yaml, e
        ))),
    }
}

/// Converts a loaded dict-object into a specified type of YamlConfigDocument if it's header matches.
///
/// :param doc_dict: source dictionary to be converted
/// :param doc_cls: instance of YamlConfigDocument to be created
/// :param ref_path_in_repo: Path of this document that should be created inside of the repositories
/// :param parent: parent document
/// :return: instance of YamlConfigDocument containing doc_dict without the header
pub(crate) fn dict_to_doc_cls(
    py: Python,
    doc_dict: YcdDict,
    doc_cls: &PyType,
    absolute_path: &str,
    ref_path_in_repo: &str,
    parent: PyYamlConfigDocument,
) -> PyResult<PyYamlConfigDocument> {
    let buf = PathBuf::from(absolute_path);
    let vrt = buf.absolutize_virtually("/")?;
    let absolute_path: String = vrt.to_str().as_ref().unwrap().to_string();
    let parent_ref = parent.borrow(py);
    let header = doc_cls.getattr("header")?.call0()?;
    let header: &str = header.extract()?;
    if doc_dict.contains_key(header) {
        let new_abs_paths: Vec<String> = [absolute_path]
            .into_iter()
            .chain(parent_ref.absolute_paths.clone())
            .collect();
        return construct_new_ycd(
            py,
            doc_cls,
            [
                doc_cls.to_object(py),
                doc_dict.get(header).unwrap().to_object(py),
                ref_path_in_repo.into_py(py),
                parent.to_object(py),
                parent_ref.already_loaded_docs.to_object(py),
                new_abs_paths.into_py(py),
            ],
        );
    }

    Err(InvalidHeaderError::new_err(format!(
        "Subdocument of type {} (path: {}) has invalid header.",
        doc_cls.getattr("__name__")?,
        ref_path_in_repo
    )))
}

/// Loads a document referenced ($ref) in a YamlConfigDocument
///
/// :param document: The document
/// :param lookup_paths: Paths to the repositories, as stored in the configuration documents
pub(crate) fn load_referenced_document(
    py: Python,
    document: PyYamlConfigDocument,
    lookup_paths: &[String],
) -> PyResult<Vec<PyYamlConfigDocument>> {
    let doc_ref: PyRef<YamlConfigDocument> = document.borrow(py);
    let ref_path_in_repo;
    if let YString(path) = doc_ref.doc.get(REF).unwrap() {
        ref_path_in_repo = path_in_repo(&doc_ref.path, path);
        if ref_path_in_repo.starts_with("./") || ref_path_in_repo.starts_with("../") {
            // Invalid path
            return Ok(vec![]);
        }
    } else {
        // Invalid path
        return Ok(vec![]);
    }
    let doc_cls: Py<PyType> = document.getattr(py, "__class__")?.extract(py)?;
    // error handling with nested iterators/vectors involved sure is readable.
    let mut out: Vec<PyYamlConfigDocument> = Vec::with_capacity(100);
    for absolute_path in absolute_paths(&ref_path_in_repo, lookup_paths)? {
        let dicts = load_dicts(&absolute_path)?;
        match dicts
            .into_iter()
            .map(|doc_dict| {
                dict_to_doc_cls(
                    py,
                    doc_dict,
                    doc_cls.as_ref(py),
                    &absolute_path,
                    &ref_path_in_repo,
                    document.clone_ref(py),
                )
            })
            .collect::<PyResult<Vec<PyYamlConfigDocument>>>()
        {
            Ok(mut inner_vec) => out.append(&mut inner_vec),
            Err(e) => return Err(e),
        };
    }
    Ok(out)
}

#[inline]
pub(crate) fn construct_new_ycd<T, U>(
    py: Python,
    cls: &PyType,
    in_args: impl IntoIterator<Item = T, IntoIter = U>,
) -> PyResult<PyYamlConfigDocument>
where
    T: ToPyObject,
    U: ExactSizeIterator<Item = T>,
{
    let args = PyTuple::new(py, in_args);
    let slf: &PyAny = cls.getattr("__new__")?.call1(args)?;
    Ok(slf.extract::<Py<YamlConfigDocument>>()?.into())
}
