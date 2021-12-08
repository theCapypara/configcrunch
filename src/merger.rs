use std::mem::take;
use pyo3::exceptions;
pub use pyo3::prelude::*;
use pyo3::types::PyType;
use crate::conv::{PyYamlConfigDocument, YcdDict, YcdList, YcdValueType};
use crate::conv::YcdValueType::{Dict, List, Ycd, YString};
use crate::{construct_new_ycd, InvalidRemoveError, load_referenced_document, REF, ReferencedDocumentNotFound, REMOVE, REMOVE_FROM_LIST_PREFIX, YamlConfigDocument};

/// Removes the $remove:: marker from all lists in doc.
pub fn delete_remove_markers(py: Python, doc: YcdValueType) -> PyResult<YcdValueType> {
    match doc {
        Ycd(mut v) => {
            let doc = take(&mut v.doc);
            match delete_remove_markers(py, Dict(doc))? {
                Dict(ndoc) => {
                    v.doc = ndoc;
                    Ok(Ycd(v))
                }
                _ => Err(exceptions::PyRuntimeError::new_err("Logic error while trying to remove delete markers."))
            }

        }
        Dict(v) => {
            match v.into_iter()
                .filter(|(_k, v)| match v {
                    YString(vs) => vs != REMOVE,
                    _ => true
                })
                .map(|(k, v)| {
                match delete_remove_markers(py, v) {
                    Ok(nv) => Ok((k, nv)),
                    Err(e) => Err(e)
                }
            }).collect::<PyResult<YcdDict>>() {
                Ok(vf) => Ok(Dict(vf)),
                Err(e) => Err(e)
            }
        }
        List(v) => {
            match v.into_iter()
                .filter(|v| match v {
                    // Remove all $remove:: entries
                    YString(vs) => !vs.starts_with(REMOVE_FROM_LIST_PREFIX),
                    _ => true
                }).map(|v| delete_remove_markers(py, v))
            .collect::<PyResult<Vec<YcdValueType>>>() {
                Ok(vf) => Ok(List(vf)),
                Err(e) => Err(e)
            }
        }
        YString(v) => {
            if v == REMOVE {
                Err(InvalidRemoveError::new_err("Tried to remove a node at an unexpected position"))
            } else {
                Ok(YString(v))
            }
        }
        _ => Ok(doc)
    }
}

/// Recursive merging step of merge_documents
//
//  :param target_node: Node to MERGE INTO
//  :param source_node: Node to MERGE FROM
//  :return: Merge result
fn merge_documents_recursion(py: Python, target_node: YcdValueType, source_node: YcdValueType) -> PyResult<YcdValueType> {
    match &source_node {
        Ycd(_) =>
            if let Ycd(t) = target_node {
                if let Ycd(s) = source_node {
                    // IS YCD IN SOURCE AND TARGET
                    return Ok(Ycd(merge_documents(py, s, t.clone_ref(py))?));
                }
                panic!(); // This is impossible.
            }
        Dict(_) =>
            if let Dict(t) = target_node {
                if let Dict(s) = source_node {
                    // IS DICT IN SOURCE AND TARGET
                    return match s.into_iter()
                            .filter(|(k, v)| match v {
                                YString(v) => !(k == REF && v == REMOVE),
                                _ => true
                            })
                            .map(|(k, v)| {
                                if t.contains_key(&k) {
                                    match merge_documents_recursion(py, t.get(&k).unwrap().clone(), v) {
                                        Ok(ov) => Ok((k, ov)),
                                        Err(e) => Err(e)
                                    }
                                } else {
                                    Ok((k, v))
                                }
                            })
                            .collect::<PyResult<YcdDict>>() {
                        Ok(ovv) => Ok(Dict(ovv)),
                        Err(e) => Err(e)
                    };
                };
                panic!(); // This is impossible.
            }
        List(_) =>
            if let List(t) = target_node {
                if let List(s) = source_node {
                    let removes: Vec<String> = t.iter()
                        .filter(|&v| match v {
                            YString(v) => v.starts_with(REMOVE_FROM_LIST_PREFIX),
                            _ => false
                        })
                        .map(|v| match v {
                            YString(v) => v.splitn(2, REMOVE_FROM_LIST_PREFIX).last().unwrap().to_string(),
                            _ => panic!("")
                        })
                        .collect();
                    return Ok(List(t.into_iter().chain(s.into_iter())
                        .filter(|v| match v {
                            YString(v) => !removes.contains(v),
                            _ => true
                        })
                        .collect::<YcdList>()
                    ));
                }
                panic!(); // This is impossible.
            }
        _ => {}
    }
    //     # IS SCALAR IN BOTH (or just in SOURCE)
    Ok(source_node)
}

/// Merges two YamlConfigDocuments.
/// :param target: Target document - this document will be changed,
///                it will contain the result of merging target into source.
/// :param source: Source document to base merge on
pub fn merge_documents(py: Python, target: PyYamlConfigDocument, source: PyYamlConfigDocument) -> PyResult<PyYamlConfigDocument> {
    let mut target_doc = target.extract(py)?;
    let source_doc = source.extract(py)?;
    match merge_documents_recursion(py, Dict(source_doc.doc.clone()), Dict(target_doc.doc))? {
        Dict(newdoc) => target_doc.doc = newdoc,
        _ => return Err(exceptions::PyRuntimeError::new_err("Invalid state while merging documents."))
    }
    target_doc.already_loaded_docs.as_mut().unwrap().extend(source_doc.already_loaded_docs.as_ref().unwrap().iter().cloned());
    let targets_before = target_doc.absolute_paths.clone();
    target_doc.absolute_paths.extend(source_doc.absolute_paths.iter()
        .filter(|&v| targets_before.contains(v))
        .map(|v| v.to_string())
    );
    Ok(target)
}

/// Resolve the $ref entry at the beginning of the document body and merge with referenced documents
/// (changes this document in place).
/// May also be extended by subclasses to include sub-document resolving.
///
/// :param doc: Document to work on
/// :param lookup_paths: Paths to the repositories, where referenced should be looked up.
pub fn resolve_and_merge(py: Python, mut pydoc: PyYamlConfigDocument, lookup_paths: &[String]) -> PyResult<PyYamlConfigDocument> {
    let mut doc: YamlConfigDocument = pydoc.extract(py)?;
    if doc.doc.contains_key(REF) {
        // Resolve references
        let mut prev_referenced_doc: Option<PyYamlConfigDocument> = None;
        for mut referenced_doc in load_referenced_document(py, pydoc.clone_ref(py), lookup_paths)? {
            if let Some(pd) = prev_referenced_doc {
                // Merge referenced docs
                referenced_doc = merge_documents(py, referenced_doc.clone_ref(py), pd)?;
            }
            prev_referenced_doc = Some(referenced_doc);
        }
        if prev_referenced_doc.is_none() {
            return if doc.absolute_paths.is_empty() {
                Err(ReferencedDocumentNotFound::new_err(format!(
                    "Referenced document {} not found. Requested by a document at {}",
                    doc.doc.get(REF).unwrap(), doc.absolute_paths[0]
                )))
            } else {
                Err(ReferencedDocumentNotFound::new_err(format!(
                    "Referenced document {} not found.",
                    doc.doc.get(REF).unwrap()
                )))
            }
        }
        // Resolve entire referenced docs
        let mut prev_referenced_doc = prev_referenced_doc.unwrap();
        prev_referenced_doc = resolve_and_merge(py, prev_referenced_doc, lookup_paths)?;
        // Merge content of current doc into referenced doc (and execute $remove's on the way)
        pydoc = merge_documents(py, pydoc prev_referenced_doc)?;
        doc: YamlConfigDocument = pydoc.extract(py)?;
        // Remove $ref entry
        doc.doc.remove(REF);
        println!("CONTAINS REF = {:?}", doc.doc);
    }
    Ok(pydoc)
}

#[pyfunction]
/// Load a subdocument of a specific type. This will convert the dict at this position
/// into a YamlConfigDocument with the matching type and perform resolve_and_merge_references
/// on it.
///
/// :param doc: Dictionary with data to convert. Can also already be a document of the target type.
/// :param source_doc: Parent document
/// :param doc_clss: Class that is expected from the subdocument (target class)
/// :param lookup_paths: Paths to the repositories, where referenced should be looked up.
pub fn load_subdocument(py: Python, doc: PyObject, source_doc_py: Py<YamlConfigDocument>, doc_clss: &PyType, _lookup_paths: Vec<String>) -> PyResult<Py<YamlConfigDocument>> {
    // doc: 'Union[dict, YamlConfigDocument]'
    let source_doc: YamlConfigDocument = source_doc_py.extract(py)?;
    let mut ycd: PyYamlConfigDocument;
    match doc.extract(py).ok() {
        None => {
            ycd = construct_new_ycd(py, doc_clss, [
                doc_clss.to_object(py),
                doc, source_doc.path.clone().into_py(py),
                source_doc_py.to_object(py), source_doc.already_loaded_docs.clone().into_py(py),
                source_doc.absolute_paths.into_py(py)
            ])?;
        },
        Some(o) => ycd = o
    }
    // TODO
    Ok(source_doc_py)
    //ycd.resolve_and_merge_references(py, lookup_paths)
}

/// Recursively removes all YamlConfigDocuments and replaces them by their doc dictionary.
pub fn recursive_docs_to_dicts(input: YcdValueType, py: Python) -> PyResult<YcdValueType> {
    match input {
        Ycd(v) => recursive_docs_to_dicts(Dict(v.doc), py),
        Dict(v) => match v.into_iter().map(|(k, v)| match recursive_docs_to_dicts(v, py) {
            Ok(vv) => Ok((k, vv)),
            Err(e) => Err(e)
        }).collect::<PyResult<YcdDict>>() {
            Ok(v) => Ok(Dict(v)),
            Err(e) => Err(e)
        }
        List(v) => match v.into_iter().map(|v| recursive_docs_to_dicts(v, py)).collect::<PyResult<Vec<YcdValueType>>>() {
            Ok(v) => Ok(List(v)),
            Err(e) => Err(e)
        },
        _ => Ok(input)
    }
}
