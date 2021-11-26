from typing import TypeVar, Type

from configcrunch import YamlConfigDocument, merge_documents

T = TypeVar('T', bound=YamlConfigDocument)


def load_multiple_yml(doc_type: Type[T], *in_args: str) -> T:
    """
    Loads (one or) multiple YAML files (paths specified by *args) into the
    given YamlConfigDocument model.
    The documents are merged as if the rightmost document "$ref"'ed the document left to it, etc.
    until all documents are merged.  However ``resolve_and_merge_references`` is not called on the base model;
    ``merge_documents`` is used instead directly.
    """
    doc = None
    if len(in_args) < 1:
        raise TypeError("At least one document path must be passed.")
    args = list(reversed(in_args))
    while len(args) > 0:
        new_doc = doc_type.from_yaml(args.pop())
        if doc is None:
            doc = new_doc
        else:
            merge_documents(new_doc, doc)
            doc = new_doc
    assert doc
    return doc  # type: ignore

