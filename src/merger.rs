use crate::conv::YcdValueType::{Dict, List, YString, Ycd};
use crate::conv::{PyYamlConfigDocument, YcdDict, YcdList, YcdValueType};
use crate::{
    construct_new_ycd, load_referenced_document, InvalidRemoveError, ReferencedDocumentNotFound,
    YamlConfigDocument, REF, REMOVE, REMOVE_FROM_LIST_PREFIX,
};
use pyo3::exceptions;
pub(crate) use pyo3::prelude::*;
use pyo3::types::PyType;
use std::collections::hash_map::Entry;
use std::iter::Peekable;
use std::mem::take;
use std::str::Split;

#[derive(FromPyObject)]
pub(crate) struct SubdocSpec(String, Py<PyType>); // path spec, type

impl SubdocSpec {
    pub(crate) fn replace_at<C>(&self, from: &mut YcdDict, cb: C, py: Python) -> PyResult<()>
    where
        C: Fn(&mut YcdValueType) -> PyResult<YcdValueType>,
    {
        let multiple = self.0.ends_with("[]");
        let path: Split<char>;
        let s;
        if multiple {
            s = self.0.chars().take(self.0.len() - 2).collect::<String>();
            path = s.split('/');
        } else {
            path = self.0.split('/');
        }
        Self::replace_at_impl(path.peekable(), from, cb, multiple, py)?;
        Ok(())
    }
    fn replace_at_impl<'s, C, P>(
        mut path: Peekable<P>,
        mut from: &mut YcdDict,
        cb: C,
        multiple: bool,
        py: Python,
    ) -> PyResult<()>
    where
        C: Fn(&mut YcdValueType) -> PyResult<YcdValueType>,
        P: Iterator<Item = &'s str>,
    {
        let mut run_at_least_once = false;
        while let Some(k) = path.next() {
            run_at_least_once = true;
            match path.peek() {
                None => match from.entry(k.to_string()) {
                    Entry::Occupied(mut oe) => {
                        if multiple {
                            match oe.get_mut() {
                                Dict(dobj) => *dobj = dobj
                                        .iter_mut()
                                        .map(|(k,v)| match cb(v) {
                                            Ok(nv) => Ok((k.clone(), {
                                                match nv {
                                                    Ycd(nvycd) => {
                                                        // Insert a $name system key to all documents in a dict, which contain the dict key.
                                                        nvycd.borrow_mut(py).doc.insert("$name".to_string(), YString(k.to_string()));
                                                        Ycd(nvycd)
                                                    }
                                                    _ => nv
                                                }
                                            })),
                                            Err(e) => Err(e)
                                        })
                                        .collect::<PyResult<YcdDict>>()?,
                                List(lobj) => *lobj = lobj
                                        .iter_mut()
                                        .map(&cb)
                                        .collect::<PyResult<YcdList>>()?,
                                YString(s) => if s != REMOVE {
                                    return Err(exceptions::PyValueError::new_err(format!("Invalid path in subdocument patterns: Invalid reference: {:?}.", oe)))
                                },
                                _ => return Err(exceptions::PyValueError::new_err(format!("Invalid path in subdocument patterns: Invalid reference: {:?}.", oe)))
                            }
                        } else {
                            let w = oe.get_mut();
                            *w = cb(w)?
                        }
                    }
                    Entry::Vacant(_ve) => return Ok(())
                }
                Some(_) => match from.get_mut(k) {
                    None => return Err(exceptions::PyValueError::new_err(
                        format!("Invalid path in subdocument patterns: Not found (expected a dict at {:?}, got nothing).", k)
                    )),
                    Some(v) => match v {
                        Dict(vv) => from = vv,
                        _ => return Err(exceptions::PyValueError::new_err(
                            format!("Invalid path in subdocument patterns: Not found (expected a dict at {:?}, got {:?}).", k, v)
                        ))
                    }
                }
            }
        }
        if run_at_least_once {
            Ok(())
        } else {
            Err(exceptions::PyValueError::new_err(
                "Invalid path in subdocument patterns: Path must not be empty.",
            ))
        }
    }
}

#[pyfunction(name = "_test__subdoc_specs")]
pub(crate) fn test_subdoc_specs(
    py: Python,
    path: String,
    typ: Py<PyType>,
    mut input: YcdDict,
    replace_with: YcdValueType,
) -> PyResult<(YcdDict, Py<PyType>)> {
    let spec = SubdocSpec(path, typ);
    spec.replace_at(&mut input, |_| Ok(replace_with.clone()), py)?;
    Ok((input, spec.1))
}

/// Removes the $remove:: marker from all lists in doc.
pub(crate) fn delete_remove_markers(py: Python, doc: YcdValueType) -> PyResult<YcdValueType> {
    match doc {
        Ycd(v) => {
            let vrc = v.clone_ref(py);
            let mut vmut = vrc.borrow_mut(py);
            let doc = take(&mut vmut.doc);
            match delete_remove_markers(py, Dict(doc))? {
                Dict(ndoc) => {
                    vmut.doc = ndoc;
                    Ok(Ycd(v))
                }
                _ => Err(exceptions::PyRuntimeError::new_err(
                    "Logic error while trying to remove delete markers.",
                )),
            }
        }
        Dict(v) => {
            match v
                .into_iter()
                .filter(|(_k, v)| match v {
                    YString(vs) => vs != REMOVE,
                    _ => true,
                })
                .map(|(k, v)| match delete_remove_markers(py, v) {
                    Ok(nv) => Ok((k, nv)),
                    Err(e) => Err(e),
                })
                .collect::<PyResult<YcdDict>>()
            {
                Ok(vf) => Ok(Dict(vf)),
                Err(e) => Err(e),
            }
        }
        List(v) => {
            let mut removes: Vec<String> = Vec::with_capacity(v.len());
            for vy in v.iter() {
                if let YString(vs) = vy {
                    if let Some(stripped) = vs.strip_prefix(REMOVE_FROM_LIST_PREFIX) {
                        removes.push(stripped.to_string())
                    }
                }
            }
            Ok(List(
                v.into_iter()
                    .filter(|v| match v {
                        // Remove all $remove:: entries
                        YString(vs) => {
                            !vs.starts_with(REMOVE_FROM_LIST_PREFIX) && !removes.contains(vs)
                        }
                        _ => true,
                    })
                    .collect::<Vec<YcdValueType>>(),
            ))
        }
        YString(v) => {
            if v == REMOVE {
                debug_assert!(false, "Tried to remove a node at an unexpected position");
                Err(InvalidRemoveError::new_err(
                    "Tried to remove a node at an unexpected position",
                ))
            } else {
                Ok(YString(v))
            }
        }
        _ => Ok(doc),
    }
}

/// Recursive merging step of merge_documents
//
//  :param target_node: Node to MERGE INTO
//  :param source_node: Node to MERGE FROM
//  :return: Merge result
fn merge_documents_recursion(
    py: Python,
    target_node: YcdValueType,
    source_node: YcdValueType,
) -> PyResult<YcdValueType> {
    match &source_node {
        Ycd(_) => {
            if let Ycd(t) = target_node {
                if let Ycd(s) = source_node {
                    // IS YCD IN SOURCE AND TARGET
                    return Ok(Ycd(merge_documents(py, s, t.clone_ref(py))?));
                }
                panic!(); // This is impossible.
            }
        }
        Dict(_) => {
            if let Dict(mut t) = target_node {
                if let Dict(s) = source_node {
                    // IS DICT IN SOURCE AND TARGET
                    t.extend(
                        s.into_iter()
                            .map(|(k, v)| {
                                if t.contains_key(&k) {
                                    match merge_documents_recursion(
                                        py,
                                        t.get(&k).unwrap().clone(),
                                        v,
                                    ) {
                                        Ok(ov) => Ok((k, ov)),
                                        Err(e) => Err(e),
                                    }
                                } else {
                                    Ok((k, v))
                                }
                            })
                            .collect::<PyResult<YcdDict>>()?,
                    );
                    return Ok(Dict(t));
                };
                panic!(); // This is impossible.
            }
        }
        List(_) => {
            if let List(t) = target_node {
                if let List(s) = source_node {
                    let removes: Vec<String> = t
                        .iter()
                        .filter(|&v| match v {
                            YString(v) => v.starts_with(REMOVE_FROM_LIST_PREFIX),
                            _ => false,
                        })
                        .map(|v| match v {
                            YString(v) => v
                                .splitn(2, REMOVE_FROM_LIST_PREFIX)
                                .last()
                                .unwrap()
                                .to_string(),
                            _ => panic!(""),
                        })
                        .collect();
                    return Ok(List(
                        t.into_iter()
                            .chain(s)
                            .filter(|v| match v {
                                YString(v) => !removes.contains(v),
                                _ => true,
                            })
                            .collect::<YcdList>(),
                    ));
                }
                panic!(); // This is impossible.
            }
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
pub(crate) fn merge_documents(
    py: Python,
    target: PyYamlConfigDocument,
    source: PyYamlConfigDocument,
) -> PyResult<PyYamlConfigDocument> {
    let targetrc = target.clone_ref(py);
    let mut target_doc = target.borrow_mut(py);
    let source_doc = source.borrow(py);
    match merge_documents_recursion(
        py,
        Dict(source_doc.doc.clone()),
        Dict(take(&mut target_doc.doc)),
    )? {
        Dict(newdoc) => target_doc.doc = newdoc,
        _ => {
            return Err(exceptions::PyRuntimeError::new_err(
                "Invalid state while merging documents.",
            ))
        }
    }
    target_doc.already_loaded_docs.as_mut().unwrap().extend(
        source_doc
            .already_loaded_docs
            .as_ref()
            .unwrap()
            .iter()
            .cloned(),
    );
    let targets_before = target_doc.absolute_paths.clone();
    target_doc.absolute_paths.extend(
        source_doc
            .absolute_paths
            .iter()
            .filter(|&v| !targets_before.contains(v))
            .map(|v| v.to_string()),
    );
    Ok(targetrc)
}

/// Resolve the $ref entry at the beginning of the document body and merge with referenced documents
/// (changes this document in place).
/// May also be extended by subclasses to include sub-document resolving.
///
/// :param doc: Document to work on
/// :param lookup_paths: Paths to the repositories, where referenced should be looked up.
pub(crate) fn resolve_and_merge(
    py: Python,
    pydoc: PyYamlConfigDocument,
    lookup_paths: &[String],
) -> PyResult<PyYamlConfigDocument> {
    let mut pydocrc = pydoc.clone_ref(py);
    let doc: PyRef<YamlConfigDocument> = pydoc.borrow(py);
    match doc.doc.get(REF) {
        Some(YString(x)) => {
            if x == REMOVE {
                return Ok(pydocrc);
            }
        }
        Some(_) => {}
        None => return Ok(pydocrc),
    };
    drop(doc);
    // Resolve references
    let mut prev_referenced_doc: Option<PyYamlConfigDocument> = None;
    for mut referenced_doc in load_referenced_document(py, pydocrc.clone_ref(py), lookup_paths)? {
        if let Some(pd) = prev_referenced_doc {
            // Merge referenced docs
            referenced_doc = merge_documents(py, referenced_doc.clone_ref(py), pd)?;
        }
        prev_referenced_doc = Some(referenced_doc);
    }
    if prev_referenced_doc.is_none() {
        let doc: PyRef<YamlConfigDocument> = pydoc.borrow(py);
        return if doc.absolute_paths.is_empty() {
            Err(ReferencedDocumentNotFound::new_err(format!(
                "Referenced document {} not found. Requested by a document at {}",
                doc.doc.get(REF).unwrap(),
                doc.absolute_paths[0]
            )))
        } else {
            Err(ReferencedDocumentNotFound::new_err(format!(
                "Referenced document {} not found.",
                doc.doc.get(REF).unwrap()
            )))
        };
    }
    // Resolve entire referenced docs
    let mut prev_referenced_doc = prev_referenced_doc.unwrap();
    prev_referenced_doc = resolve_and_merge(py, prev_referenced_doc, lookup_paths)?;
    // Merge content of current doc into referenced doc (and execute $remove's on the way)
    pydocrc = merge_documents(py, pydocrc, prev_referenced_doc)?;
    // Remove $ref entry
    pydocrc.borrow_mut(py).doc.remove(REF);
    Ok(pydocrc)
}

/// Load a subdocument of a specific type. This will convert the dict at this position
/// into a YamlConfigDocument with the matching type and perform resolve_and_merge_references
/// on it.
pub(crate) fn load_subdocument(
    py: Python,
    doc: &mut YcdValueType,
    args: [PyObject; 4],
    doc_clss: Py<PyType>,
    lookup_paths: &[String],
) -> PyResult<YcdValueType> {
    let ycd = match doc {
        Ycd(v) => v.clone_ref(py),
        Dict(d) => construct_new_ycd(py, doc_clss.extract(py)?, [&[
                doc_clss.to_object(py),
                d.to_object(py),
            ][..], &args[..]].concat())?,
        YString(s) => return if s == REMOVE {
            Ok(YString(REMOVE.to_string()))
        } else {
            Err(exceptions::PyValueError::new_err(format!("Invalid path in subdocument: Invalid reference where a dict or document was expected: {:?}.", s)))
        },
        _ => return Err(exceptions::PyValueError::new_err(format!("Invalid path in subdocument: Invalid reference where a dict or document was expected: {:?}.", doc)))
    };
    Ok(Ycd(YamlConfigDocument::resolve_and_merge_references(
        ycd.into(),
        py,
        lookup_paths.to_vec(),
    )?
    .into()))
}

/// Loads all subdocuments for doc, according to the specification.
/// Calls load_subdocument for each entry, see for details.
pub(crate) fn load_subdocuments(
    py: Python,
    doc: PyYamlConfigDocument,
    specs: Vec<SubdocSpec>,
    lookup_paths: &[String],
) -> PyResult<()> {
    let mut doc_borrow = doc.borrow_mut(py);
    let args = [
        doc_borrow.path.clone().into_py(py),
        doc.to_object(py),
        doc_borrow.already_loaded_docs.clone().into_py(py),
        doc_borrow.absolute_paths.clone().into_py(py),
    ];
    for spec in specs {
        spec.replace_at(
            &mut doc_borrow.doc,
            |target| load_subdocument(py, target, args.clone(), spec.1.clone_ref(py), lookup_paths),
            py,
        )?;
    }
    Ok(())
}

/// Recursively removes all YamlConfigDocuments and replaces them by their doc dictionary.
pub(crate) fn recursive_docs_to_dicts(input: YcdValueType, py: Python) -> PyResult<YcdValueType> {
    match input {
        Ycd(v) => recursive_docs_to_dicts(Dict(v.borrow(py).doc.clone()), py),
        Dict(v) => match v
            .into_iter()
            .map(|(k, v)| match recursive_docs_to_dicts(v, py) {
                Ok(vv) => Ok((k, vv)),
                Err(e) => Err(e),
            })
            .collect::<PyResult<YcdDict>>()
        {
            Ok(v) => Ok(Dict(v)),
            Err(e) => Err(e),
        },
        List(v) => match v
            .into_iter()
            .map(|v| recursive_docs_to_dicts(v, py))
            .collect::<PyResult<Vec<YcdValueType>>>()
        {
            Ok(v) => Ok(List(v)),
            Err(e) => Err(e),
        },
        _ => Ok(input),
    }
}
