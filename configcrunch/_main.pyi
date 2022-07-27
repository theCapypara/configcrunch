# See the Rust source code for docstrings, query them via Python or see the documentation..
from __future__ import annotations

from abc import abstractmethod
from typing import List, Callable, Type, Optional, Union, TypeVar, final, Tuple, Any, ContextManager

from schema import Schema

from configcrunch import variable_helper


T = TypeVar('T', bound=YamlConfigDocument)


class ConfigcrunchError(Exception): ...
class ReferencedDocumentNotFound(ConfigcrunchError): ...
class CircularDependencyError(ConfigcrunchError): ...
class VariableProcessingError(ConfigcrunchError): ...
class InvalidDocumentError(ConfigcrunchError): ...
class InvalidHeaderError(InvalidDocumentError):...
class InvalidRemoveError(InvalidDocumentError): ...


def load_multiple_yml(doc_type: Type[T], *in_args: str) -> T: ...


class YamlConfigDocument:
    path: Optional[str]
    parent_doc: Optional[YamlConfigDocument]
    absolute_paths: List[str]
    
    def __init__(
            self, document: YamlConfigDocument, path: Optional[str], parent_doc: Optional[YamlConfigDocument]
    ):
        """Manual constructor. It's recommended to use from_yaml or from_dict instead."""
        ...

    @property
    def doc(self) -> dict:
        """READONLY representation of the document, this is also accessible via __getitem__ etc. ONLY after calling freeze()"""
        ...

    @classmethod
    @final
    def from_yaml(cls, path_to_yaml: str) -> YamlConfigDocument: ...
    @classmethod
    @final
    def from_dict(cls, dict: dict) -> YamlConfigDocument: ...
    @final
    def freeze(self): ...
    @classmethod
    @abstractmethod
    def header(cls) -> str: ...
    @classmethod
    @abstractmethod
    def schema(cls) -> Schema: ...
    @classmethod
    @abstractmethod
    def subdocuments(cls) -> List[Tuple[str, Type[YamlConfigDocument]]]: ...
    def validate(self) -> bool: ...
    @final
    def resolve_and_merge_references(self, lookup_paths: List[str]) -> YamlConfigDocument: ...
    @final
    def process_vars(self) -> YamlConfigDocument: ...
    def process_vars_for(self, target: str, additional_helpers: List[Callable] = None) -> str: ...
    @variable_helper
    def parent(self) -> Optional[YamlConfigDocument]: ...
    def __repr__(self) -> str: ...
    def __str__(self): ...
    def error_str(self) -> str: ...
    def __len__(self): ...
    def __getitem__(self, key): ...
    def __setitem__(self, key, value): ...
    def __delitem__(self, key): ...
    def __iter__(self): ...
    def items(self): ...
    def to_dict(self): ...
    def internal_get(self, key: str) -> Any: ...
    def internal_set(self, key: str, val: Any): ...
    def internal_contains(self, key: str) -> bool: ...
    def internal_delete(self, key: str): ...
    def internal_access(self) -> ContextManager: ...
    # These DO NOT ACTUALLY EXIST on the parent object.
    def _initialize_data_before_merge(self, data: dict) -> dict:
        """
        May be used to initialize the document by adding / changing data.

        Called before doing anything else in resolve_and_merge_references.
        Use this for setting default values.
        The internal data is passed as an argument and can be mutated.
        The changed data MUST be returned again.
        """
        ...
    def _initialize_data_after_merge(self, data: dict) -> dict:
        """
        May be used to initialize the document by adding / changing data.

        Called after resolve_and_merge_references.
        Use this for setting default values.
        The internal data is passed as an argument and can be mutated.
        The changed data MUST be returned again.
        """
        ...
    def _initialize_data_after_variables(self, data: dict) -> dict:
        """
        May be used to initialize the document by adding / changing data.

        Called after process_vars.
        Use this for setting internal values based on processed values in the document.
        The internal data is passed as an argument and can be mutated.
        The changed data MUST be returned again.
        """
        ...
    def _initialize_data_after_freeze(self):
        """
        May be used to initialize the document by adding / changing data.

        Called after freeze.
        Use this for setting internal values based on processed values in the document.
        You can access the data using the self.doc property or by getting it from self (self[...]).
        """
        ...


class DocReference:
    def __init__(self, referenced_doc_type: Type[YamlConfigDocument]): ...
    def validate(self, data): ...


def _test__subdoc_specs(path: str, type: Type[Any], input: dict) -> Tuple[dict, str, Any, bool, Type[Any]]: ...
