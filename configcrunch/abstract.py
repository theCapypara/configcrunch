import inspect
import warnings
from abc import ABC, abstractmethod

import yaml
from schema import Schema,  SchemaError
from typing import List, Type, Union, Callable

from configcrunch import REF
from configcrunch.interface import IYamlConfigDocument
from configcrunch.merger import resolve_and_merge, recursive_docs_to_dicts, delete_remove_markers
from configcrunch.errors import InvalidHeaderError, CircularDependencyError, InvalidDocumentError
from configcrunch.variables import process_variables, process_variables_for

DUMP_FOR_REPR = False


def variable_helper(func):
    orig_doc = ""
    if hasattr(func, "__doc__") and func.__doc__ is not None:
        orig_doc = func.__doc__
    func.__doc__ = """.. admonition:: Variable Helper
                  
                  Can be used inside configuration files.

""" + orig_doc
    func.__is_variable_helper = True
    return func


class YamlConfigDocument(IYamlConfigDocument, ABC):
    """
    A document represented by a dictionary, that can be validated,
    can contain references to other (sub-)documents, which can be resolved,
    and variables that can be parsed.
    """
    def __init__(
            self,
            document: dict,
            path: str= None,
            parent: 'YamlConfigDocument'= None,
            already_loaded_docs: List[str]= None,
            absolute_paths=None
    ):
        """
        Constructs a YamlConfigDocument

        :param absolute_paths: absolute paths on disk to this YCD.
                               This is a list, ordered by order of merge. First entry is the last merged file
                               (the file with the first $ref in it).
        :param document: The document as a dict, without the header.
        :param path: Path of the document absolute to the configured repositories.
                     If this is not from a repo, leave at None.
        :param parent: Parent document
        :param already_loaded_docs: List of paths to already loaded documents (internal use)
        """
        if absolute_paths is None:
            absolute_paths = []
        self.doc = document
        self.path = path
        self.bound_helpers = []
        self.parent_doc = parent
        self.absolute_paths = absolute_paths

        self.__infinite_recursion_check(already_loaded_docs)
        self.__collect_bound_variable_helpers()

    @classmethod
    def from_yaml(cls, path_to_yaml: str) -> 'YamlConfigDocument':
        """
        Constructs a YamlConfigDocument from a YAML-file.

        Expects the content to be a dictionary with one key (defined in the
        header method) and it's value is the body of the document,
        validated by the schema method.

        :param path_to_yaml:
        :return:
        """
        with open(path_to_yaml, 'r') as stream:
            entire_document = yaml.safe_load(stream)
        # The document must start with a header matching it's class
        if not isinstance(entire_document, dict):
            raise InvalidDocumentError(f"The document at {path_to_yaml} is invalid")
        if cls.header() not in entire_document:
            raise InvalidHeaderError("The document does not have a valid header. Expected was: " + cls.header())
        body = entire_document[cls.header()]
        return cls(body, absolute_paths=[path_to_yaml])

    @classmethod
    @abstractmethod
    def header(cls) -> str:
        """ Header that YAML-documents must contain. """
        pass

    @classmethod
    @abstractmethod
    def schema(cls) -> Schema:
        """ Schema that the document should be validated against. """
        pass

    def validate(self) -> bool:
        """ Validates the document against the Schema. """
        self.schema().validate(self.doc)
        return True

    def _initialize_data_after_merge(self):
        """
        May be used to initialize the document by adding / changing data.

        Called after resolve_and_merge_references.
        Use this for setting default values.
        """
        pass

    def _initialize_data_after_variables(self):
        """
        May be used to initialize the document by adding / changing data.

        Called after process_vars.
        Use this for setting internal values based on processed values in the document.
        """
        pass

    def resolve_and_merge_references(self, lookup_paths: List[str]) -> 'YamlConfigDocument':
        """
        Resolve the $ref entry at the beginning of the document body and merge with referenced documents
        (changes this document in place).

        :param lookup_paths: Paths to the repositories, where referenced should be looked up.

        :final: Since 0.2.0 this function must not be extended, otherwise the $remove markers
                will not be removed. Please use _load_subdocuments to load sub-documents during the merging step instead!

        :returns: self
        """

        if 'resolve_and_merge_references' in vars(self.__class__):
            warnings.warn("resolve_and_merge_references is final and must not be extended since "
                          "Configcrunch 0.2, extend _load_subdocuments instead.", DeprecationWarning)

        resolve_and_merge(self, lookup_paths)
        self._initialize_data_after_merge()
        self._load_subdocuments(lookup_paths)
        delete_remove_markers(self)
        return self

    def _load_subdocuments(self, lookup_paths: List[str]):
        """
        Load sub-documents during the merging step.
        Override this to load custom sub-documents.
        Make sure to check if the value you are trying to load is $remove (constant REMOVE) first!
        """
        pass

    def process_vars(self) -> 'YamlConfigDocument':
        """
        Process all {{ variables }} inside this document and all sub-documents.
        All references must be resolved beforehand to work correctly (resolve_and_merge_references).
        Changes this document in place.
        """
        process_variables(self)
        self._initialize_data_after_variables()
        return self

    def process_vars_for(self, target: str, additional_helpers: List[Callable] = None) -> str:
        """
        Process all {{ variables }} inside the specified string as if it were part of this document.
        All references must be resolved beforehand to work correctly (resolve_and_merge_references).

        additional_helpers may contain additional variable helper functions to use.
        """
        return process_variables_for(self, target, additional_helpers)

    @variable_helper
    def parent(self) -> 'YamlConfigDocument':
        """
        A helper function that can be used by variable-placeholders to the get the parent document (if any is set).

        Example usage::

            something: '{{ parent().field }}'

        Example result::

            something: 'value of parent field'

        """
        if self.parent_doc:
            return self.parent_doc
        else:
            return self

    def __collect_bound_variable_helpers(self):
        """ Loads bound variable helper methods to this instance for use in jinja2 variable processing """
        for name, method in inspect.getmembers(self, predicate=inspect.ismethod):
            if hasattr(method, "__is_variable_helper"):
                self.bound_helpers.append(method)

    def __infinite_recursion_check(self, already_loaded_docs: List[str]):
        """ Infinite recursion check """
        if already_loaded_docs is not None and self.path is not None:
            if self.path in already_loaded_docs:
                raise CircularDependencyError("Infinite circular reference detected while trying to load " + self.path)
            self.already_loaded_docs = already_loaded_docs.copy()
            self.already_loaded_docs.append(self.path)
        elif already_loaded_docs is not None:
            self.already_loaded_docs = already_loaded_docs.copy()
        else:
            self.already_loaded_docs = []

    def __repr__(self) -> str:
        return str(self)

    def __str__(self):
        return self.__class__.__name__ + "(" + str(self.doc) + ")"

    def error_str(self) -> str:
        """
        Error string representation.
        This short string representation is used in Schema errors and is meant to assist in finding
        document errors. Set this to a small representation of the document, that the user can understand.
        """
        return "type " + self.__class__.__name__

    def __len__(self):
        return len(self.doc)

    def __getitem__(self, key):
        return self.doc[key]

    def __setitem__(self, key, value):
        self.doc[key] = value

    def __delitem__(self, key):
        del self.doc[key]

    def __iter__(self):
        return iter(self.doc)

    def items(self):
        return self.doc.items()

    def to_dict(self):
        return recursive_docs_to_dicts({self.header(): self.doc.copy()})


class DocReference:
    """
    For Schemas.
    Marks a reference to another YamlConfigDocument inside a schema.
    """
    def __init__(self, referenced_doc_type: Type[YamlConfigDocument]):
        self.referenced_doc_type = referenced_doc_type

    def validate(self, data):
        """
        Validates. If the subdocument still contains $ref, it is not validated further,
        please call resolve_and_merge_references. Otherwise the sub-document is expected to match
        according to it's schema.

        :param data:
        :return:
        """
        # If the reference still contains the $ref keyword, it is treated as an
        # unmerged reference and not validated further.
        if REF in data:
            return True

        if isinstance(data, self.referenced_doc_type):
            # data is a YamlConfigDocument of the expected type
            # We assume a fully merged and valid document with all values.
            try:
                data.validate()
            except SchemaError as e:
                raise SchemaError("Error parsing subdocument " + data.error_str(), e.errors)
        else:
            raise SchemaError('Expected an instance of ' + self.referenced_doc_type.__name__ + ' while validating.', [])


def ycd_representer(dumper, data):
    return dumper.represent_mapping('!' + data.__class__.__name__, data.doc)


yaml.add_multi_representer(YamlConfigDocument, ycd_representer)
