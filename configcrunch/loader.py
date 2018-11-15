"""
Loader module, contains code to actually resolve and load documents from repositories.
Should not be used outside the library.
"""

import os
from typing import TYPE_CHECKING, List, Type

import yaml

from configcrunch import REF
from configcrunch.errors import InvalidHeaderError

if TYPE_CHECKING:
    from configcrunch.abstract import YamlConfigDocument


def load_repos(lookup_paths: List[str]) -> List[str]:
    """
    Load the full absolute paths to the repositories stored on disc.
    If the paths are Git-repositories, they may be cloned first.
    :param lookup_paths:
    :return:
    """
    repo_paths = []
    for path in lookup_paths:
        if path.startswith('./') or path.startswith('.\\'):
            # relative path to project folder
            repo_paths.append(os.path.join(os.getcwd(), path[2:]))
            # TODO: use project folder instead of CWD
            # TODO: support later?
        if path.startswith('/') or path.startswith('\\'):
            # Absolute Paths
            # TODO: Windows conversion
            repo_paths.append(path)
        else:
            # TODO try git
            # TODO Should not be part of cc
            pass

    return repo_paths


def path_in_repo(base_path: str, reference_path: str) -> str:
    """
    Convert a $ref-Path into a full path absolute to the root of the repositories
    :param base_path: Path of the file that contained the $ref or None if document was not part of the repositories
    :param reference_path: Entry in $ref field.
    :return: final path inside the repositories
    """
    # TODO ../ paths
    path = reference_path.lstrip('/')
    if reference_path.startswith('./') and base_path is not None:
        # removes last / and everything after it
        current_path = '/'.join(base_path.split('/')[:-1])
        path = current_path + '/' + reference_path[2:]
    return path


def absolute_paths(ref_path_in_repo: str, lookup_paths: List[str]) -> List[str]:
    """
    Appends the paths inside repositories to the lookup_paths/repository paths, building a unique
    absolute path on the disc that is only missing the file extension.
    :param ref_path_in_repo: Path of resoruce absolute to repository root
    :param lookup_paths: Paths to the repositories, as stored in the configuration documents
    :return:
    """
    paths = []
    for absolute_repo_path in load_repos(lookup_paths):
        paths.append(absolute_repo_path + '/' + ref_path_in_repo)

    return paths


def load_dicts(path: str) -> List[dict]:
    """
    Load the actual dictionaries at path by checking if files ending in .yml/.yaml exist.
    :param path:
    :return:
    """
    doc_dicts = []

    yml_filename = path + ".yml"
    if os.path.isfile(yml_filename):
        with open(yml_filename, 'r') as stream:
            doc_dicts.append(yaml.load(stream))

    yaml_filename = path + ".yaml"
    if os.path.isfile(yaml_filename):
        with open(yaml_filename, 'r') as stream:
            doc_dicts.append(yaml.load(stream))

    return doc_dicts


def dict_to_doc_cls(
        doc_dict: dict,
        doc_cls: 'Type[YamlConfigDocument]',
        ref_path_in_repo: str,
        parent: 'YamlConfigDocument'
) -> 'YamlConfigDocument':
    """
    Converts a loaded dict-object into a specified type of YamlConfigDocument if it's header matches.
    :param doc_dict: source dictionary to be converted
    :param doc_cls: instance of YamlConfigDocument to be created
    :param ref_path_in_repo: Path of this document that should be created inside of the repositories
    :param parent: parent document
    :return: instance of YamlConfigDocument containing doc_dict without the header
    """
    # resolve document path[s]
    if doc_cls.header() in doc_dict:
        doc = doc_cls(doc_dict[doc_cls.header()], ref_path_in_repo, parent, parent.already_loaded_docs)
    else:
        raise InvalidHeaderError("Subdocument of type " + doc_cls.__name__ + " (path: " + ref_path_in_repo + ") has invalid header.")
    return doc


def load_referenced_document(document: 'YamlConfigDocument', lookup_paths: List[str]) -> 'List[YamlConfigDocument]':
    """
    Loads a document referenced ($ref) in a YamlConfigDocument
    :param document: The document
    :param lookup_paths: Paths to the repositories, as stored in the configuration documents
    :return:
    """
    docs = []
    ref_path_in_repo = path_in_repo(document.path, document[REF])
    doc_cls = document.__class__
    for absolute_path in absolute_paths(ref_path_in_repo, lookup_paths):
        for doc_dict in load_dicts(absolute_path):
            doc = dict_to_doc_cls(doc_dict, doc_cls, ref_path_in_repo, document)
            docs.append(doc)
    return docs
