from abc import ABC, abstractmethod

import yaml
from schema import Schema, SchemaError
from typing import List, Type

from configcrunch import REF
from configcrunch.interface import IYamlConfigDocument
from configcrunch.merger import resolve_and_merge


def variable_helper(funcobj):
    funcobj.__is_variable_helper = True
    return funcobj


DUMP_FOR_REPR = False


class YamlConfigDocument(IYamlConfigDocument, ABC):
    """
    A document represented by a dictionary, that can be validated,
    can contain references to other (sub-)documents, which can be resolved,
    and variables that can be parsed.
    """
    def __init__(self, document: dict, path=None):
        """
        Constructs a YamlConfigDocument
        :param document: The document as a dict, without the header.
        :param path: Path of the document absolute to the configured repositories.
                     If this is not from a repo, leave at None.
        """
        self.doc = document
        self.path = path

    @classmethod
    def from_yaml(cls, path_to_yaml: str):
        """
        Constructs a YamlConfigDocument from a YAML-file. Expects the content to be
        a dictionary with one key defined in the header method and it's value is
        the body of the document, validated by the schema method.
        :param path_to_yaml:
        :return:
        """
        with open(path_to_yaml, 'r') as stream:
            entire_document = yaml.load(stream)
        # The document must start with a header matching it's class
        if cls.header() not in entire_document:
            raise Exception("not valid header.")  # TODO better exception!
        body = entire_document[cls.header()]
        return cls(body)

    @classmethod
    @abstractmethod
    def header(cls) -> str:
        """ Returns the header that YAML-documents should contain. """
        pass

    @abstractmethod
    def schema(self) -> Schema:
        """ Returns the schema that the document should be validated against """
        pass

    def validate(self) -> bool:
        """ Validates the document against the Schema. """
        return self.schema().validate(self.doc)

    def resolve_and_merge_references(self, lookup_paths: List[str]) -> 'YamlConfigDocument':
        """
        Resolve the $ref entry at the beginning of the document body and merge with referenced documents
        (changes this document in place).
        May also be extended by subclasses to include sub-document resolving.
        :param lookup_paths: Paths to the repositories, where referenced should be looked up.
        :return:
        """
        resolve_and_merge(self, lookup_paths)
        return self

    def process_vars(self) -> 'YamlConfigDocument':
        """
        Process all {{ variables }} inside this document and all sub-documents.
        All references must be resolved beforehand to work correctly (resolve_and_merge_references).
        Changes this document in place.
        """
        pass  # todo

    @variable_helper
    def parent(self) -> 'YamlConfigDocument':
        """ A helper function that can be used by variable-placeholders to the get the parent document (if any) """
        pass  # todo

    def __repr__(self) -> str:
        return str(self)

    def __str__(self):
        return self.__class__.__name__ + "(" + str(self.doc) + ")"

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


class DocReference(object):
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
                raise SchemaError("Error parsing subdocument of type " + self.referenced_doc_type.__name__, e.errors)
        else:
            raise SchemaError('Expected an instance of ' + self.referenced_doc_type.__name__ + ' while validating.', [])


def ycd_representer(dumper, data):
    return dumper.represent_mapping('!' + data.__class__.__name__, data.doc)


yaml.add_multi_representer(YamlConfigDocument, ycd_representer)
