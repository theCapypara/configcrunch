"""
Merger module.
Contains the logic to merge loaded documents.

- resolve_and_merge may be used to resolve and merge $ref entries in documents (as used by YamlConfigDocument).
- load_subdocument may be used to load and merge sub-documents contained in YamlConfigDocuments.

"""
from typing import Union, Type, List

from configcrunch import REF, REMOVE

from typing import TYPE_CHECKING

from configcrunch.interface import IYamlConfigDocument
from configcrunch.loader import load_referenced_document
from configcrunch.errors import ReferencedDocumentNotFound, InvalidRemoveError

if TYPE_CHECKING:
    from configcrunch.abstract import YamlConfigDocument


def _merge_documents__recursion(target_node: any, source_node: any) -> any:
    """
    Recursive merging step of merge_documents

    :param target_node: Node to MERGE INTO
    :param source_node: Node to MERGE FROM
    :return: Merge result
    """
    # IS DICT IN SOURCE AND TARGET
    if isinstance(source_node, dict) and isinstance(target_node, dict):
        new_node = target_node.copy()
        for key, value in source_node.items():
            if value == REMOVE:
                if key in new_node:
                    del new_node[key]
            else:
                if key in target_node:
                    new_node[key] = _merge_documents__recursion(target_node[key], source_node[key])
                else:
                    new_node[key] = source_node[key]
        return new_node

    # IS LIST IN SOURCE AND TARGET
    elif isinstance(source_node, list) and isinstance(target_node, list):
        result = list(target_node)
        result.extend(x for x in source_node if x not in target_node)
        return result

    # IS YCD IN SOURCE AND TARGET
    elif isinstance(source_node, IYamlConfigDocument) and isinstance(target_node, IYamlConfigDocument):
        merge_documents(source_node, target_node)
        return source_node

    # IS $remove IN SOURCE AND ... IN TARGET
    if source_node == REMOVE:
        raise InvalidRemoveError("Tried to remove a node at an unexpected position")

    # IS SCALAR IN BOTH (or just in SOURCE)
    else:
        return source_node


def merge_documents(target: 'YamlConfigDocument', source: 'YamlConfigDocument') -> None:
    """
    Merges two YamlConfigDocuments.

    :param target: Target document - this document will be changed, it will contain the result of merging target into source.
    :param source: Source document to base merge on
    """
    newdoc = _merge_documents__recursion(source.doc, target.doc)
    target.doc = newdoc
    target.already_loaded_docs += source.already_loaded_docs
    target.absolute_paths += list(set(source.absolute_paths) - set(target.absolute_paths))


def resolve_and_merge(doc: 'YamlConfigDocument', lookup_paths: List[str]) -> None:
    """
    Resolve the $ref entry at the beginning of the document body and merge with referenced documents
    (changes this document in place).
    May also be extended by subclasses to include sub-document resolving.

    :param doc: Document to work on
    :param lookup_paths: Paths to the repositories, where referenced should be looked up.
    :return:
    """
    if REF in doc:
        # Resolve references
        prev_referenced_doc = None
        for referenced_doc in load_referenced_document(doc, lookup_paths):
            if prev_referenced_doc:
                # Merge referenced docs
                merge_documents(referenced_doc, prev_referenced_doc)
            prev_referenced_doc = referenced_doc
        if prev_referenced_doc is None:
            if doc.absolute_paths:
                raise ReferencedDocumentNotFound("Referenced document %s not found. Requested by a document at %s"
                                                 % (doc[REF], doc.absolute_paths[0]))
            else:
                raise ReferencedDocumentNotFound("Referenced document %s not found." % doc[REF])
        # Resolve entire referenced docs
        resolve_and_merge(prev_referenced_doc, lookup_paths)
        # Merge content of current doc into referenced doc (and execute $remove's on the way)
        merge_documents(doc, prev_referenced_doc)
        # Remove $ref entry
        del doc[REF]


def load_subdocument(
        doc: 'Union[dict, YamlConfigDocument]',
        source_doc: 'YamlConfigDocument',
        doc_clss: 'Type[YamlConfigDocument]',
        lookup_paths: List[str],
) -> 'YamlConfigDocument':
    """
    Load a subdocument of a specific type. This will convert the dict at this position
    into a YamlConfigDocument with the matching type and perform resolve_and_merge_references
    on it.

    :param doc: Dictionary with data to convert. Can also already be a document of the target type.
    :param source_doc: Parent document
    :param doc_clss: Class that is expected from the subdocument (target class)
    :param lookup_paths: Paths to the repositories, where referenced should be looked up.
    :return:
    """
    doc_obj = doc
    if not isinstance(doc, doc_clss):
        doc_obj = doc_clss(doc, source_doc.path, source_doc,
                           source_doc.already_loaded_docs, absolute_paths=source_doc.absolute_paths)

    return doc_obj.resolve_and_merge_references(lookup_paths)


def recursive_docs_to_dicts(input):
    """ Recursively removes all YamlConfigDocuments and replaces them by their doc dictionary."""
    if isinstance(input, IYamlConfigDocument):
        return recursive_docs_to_dicts(input.doc.copy())
    elif isinstance(input, dict):
        new_dict = input.copy()
        for key, val in new_dict.items():
            new_dict[key] = recursive_docs_to_dicts(val)
        return new_dict
    elif isinstance(input, list):
        new_list = []
        for item in input.copy():
            new_list.append(recursive_docs_to_dicts(item))
        return new_list
    return input
