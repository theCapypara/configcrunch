import yaml

from configcrunch._main import YamlConfigDocument, DocReference, load_subdocument, merge_documents, \
    ConfigcrunchError, ReferencedDocumentNotFound, CircularDependencyError, \
    VariableProcessingError, InvalidDocumentError, InvalidHeaderError, InvalidRemoveError
import configcrunch.advanced_loader

# Constants
REF = "$ref"
REMOVE = "$remove"
REMOVE_FROM_LIST_PREFIX = REMOVE + "::"


def variable_helper(func):
    orig_doc = ""
    if hasattr(func, "__doc__") and func.__doc__ is not None:
        orig_doc = func.__doc__
    func.__doc__ = """.. admonition:: Variable Helper

                  Can be used inside configuration files.

""" + orig_doc
    func.__is_variable_helper = True
    return func


def ycd_representer(dumper, data):
    return dumper.represent_mapping('!' + data.__class__.__name__, data.doc)


yaml.add_multi_representer(YamlConfigDocument, ycd_representer)


# Public classes and functions
__all__ = [
    'YamlConfigDocument',
    'DocReference',
    'variable_helper',
    'load_subdocument',
    'merge_documents',

    'ConfigcrunchError',
    'ReferencedDocumentNotFound',
    'CircularDependencyError',
    'VariableProcessingError',
    'InvalidDocumentError',
    'InvalidHeaderError',
    'InvalidRemoveError',

    'advanced_loader'
]
