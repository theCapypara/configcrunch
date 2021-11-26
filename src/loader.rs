use std::collections::HashMap;
use std::env::current_dir;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use pyo3::exceptions;
pub use pyo3::prelude::*;
use pyo3::types::{PyTuple, PyType};
use crate::{InvalidDocumentError, InvalidHeaderError, REF, YamlConfigDocument};
use crate::conv::{PyYamlConfigDocument, SimpleYcdValueType, YcdDict, YHashMap};
use crate::conv::YcdValueType::YString;

/// Load the full absolute paths to the repositories (lookup paths) stored on disk.
pub(crate) fn load_repos(lookup_paths: &[String]) -> Vec<String> {
    lookup_paths.iter().map(|p| to_abs_path(p)).collect()
}

fn to_abs_path(str: &str) -> String {
    let ch = str.chars().next().unwrap();
    if ch == '/' || ch == '\\' {
        return current_dir().unwrap().join(str).to_str().unwrap().to_string();
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
                "/", PathBuf::from(p).parent().unwrap().to_str().unwrap(), reference_path
            ].iter().collect();
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
pub(crate) fn absolute_paths(ref_path_in_repo: &str, lookup_paths: &[String]) -> PyResult<Vec<String>> {
    let ref_path_in_repo_cln: &str;
    match ref_path_in_repo.strip_prefix('/') {
        None => ref_path_in_repo_cln = ref_path_in_repo,
        Some(p) => ref_path_in_repo_cln = p
    }
    load_repos(lookup_paths).iter().map(|absolute_repo_path| {
        // TODO: Is this safe?
        Ok(format!("{}/{}", absolute_repo_path, ref_path_in_repo_cln))
    }).collect::<PyResult<Vec<String>>>()
}


/// Load the actual dictionaries at path by checking if files ending in .yml/.yaml exist.
pub(crate) fn load_dicts(path: &str) -> PyResult<Vec<YcdDict>> {
    let mut doc_dicts: Vec<YcdDict> = Vec::with_capacity(2);
    let p_yml = format!("{}.yml", path);
    let p_yaml = format!("{}.yaml", path);
    if let Ok(e) = fs::try_exists(&p_yml) {
        if e {
            doc_dicts.push(load_yaml_file(&p_yml)?);
        }
    }
    if let Ok(e) = fs::try_exists(&p_yaml) {
        if e {
            doc_dicts.push(load_yaml_file(&p_yaml)?);
        }
    }

    Ok(doc_dicts)
}

pub(crate) fn load_yaml_file(path_to_yaml: &str) -> PyResult<YcdDict> {
    let file;
    match File::open(path_to_yaml) {
        Ok(v) => file = v,
        Err(e) => return Err(InvalidDocumentError::new_err(format!("Unable to open YAML file {}: {:?}", path_to_yaml, e)))
    };

    match serde_yaml::from_reader::<File, HashMap<String, SimpleYcdValueType>>(file) {
        Ok(v) => Ok(YHashMap(v).into()),
        Err(e) => Err(InvalidDocumentError::new_err(format!("Unable to read YAML file {}: {:?}", path_to_yaml, e)))
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
    py: Python, doc_dict: YcdDict, doc_cls: &PyType, absolute_path: &str, ref_path_in_repo: &str, parent: &YamlConfigDocument, parent_pyobj: PyObject,
) -> PyResult<PyYamlConfigDocument> {
    let header = doc_cls.getattr("header")?.call0()?;
    let header: &str = header.extract()?;
    if doc_dict.contains_key(header) {
        let new_abs_paths: Vec<String> = [absolute_path.to_string()]
            .into_iter().chain(parent.absolute_paths.clone().into_iter())
            .collect();
        let new_doc = construct_new_ycd(py, doc_cls, [
            doc_cls.to_object(py),
            doc_dict.get(header).unwrap().to_object(py),
            ref_path_in_repo.into_py(py),
            parent_pyobj,
            (&parent.already_loaded_docs).to_object(py),
            new_abs_paths.into_py(py)
        ])?;

        return Ok(new_doc.extract::<Py<YamlConfigDocument>>(py)?.into());
    }

    Err(InvalidHeaderError::new_err(
        format!(
            "Subdocument of type {} (path: {}) has invalid header.",
            doc_cls.getattr("__name__")?, ref_path_in_repo
        )))
}

/// Loads a document referenced ($ref) in a YamlConfigDocument
///
/// :param document: The document
/// :param lookup_paths: Paths to the repositories, as stored in the configuration documents
pub(crate) fn load_referenced_document(
    py: Python, document: PyYamlConfigDocument, lookup_paths: &[String]
) -> PyResult<Vec<PyYamlConfigDocument>> {
    let doc_ref = document.extract(py)?;
    let ref_path_in_repo;
    if let YString(path) = doc_ref.doc.extract(py)?.get(REF).unwrap() {
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
        if let Ok(dicts) = load_dicts(&absolute_path) {
            match dicts.into_iter().map(|doc_dict| dict_to_doc_cls(
                py, doc_dict, doc_cls.as_ref(py), &absolute_path, &ref_path_in_repo, &doc_ref,
                document.to_object(py)
            )).collect() {
                Ok(mut inner_vec) => out.append(&mut inner_vec),
                Err(e) => return Err(e)
            };
        }
    }
    Ok(out)
}

#[inline]
pub(crate) fn construct_new_ycd<T, U>(py: Python, cls: &PyType, in_args: impl IntoIterator<Item = T, IntoIter = U>) -> PyResult<PyObject>
    where
        T: ToPyObject,
        U: ExactSizeIterator<Item = T>,
{
    let args = PyTuple::new(py, in_args);
    let slf: &PyAny = cls.getattr("__new__")?.call1(args)?;
    Ok(slf.to_object(py))
}
