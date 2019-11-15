"""
Classes that implement YamlConfigDocument and represent
YAML documents for the tests.
"""
from typing import List

from schema import Schema, Optional

from configcrunch import YamlConfigDocument, DocReference, load_subdocument, REMOVE
from configcrunch.abstract import variable_helper


class Base(YamlConfigDocument):
    """
    Base test document. Example:

    base:
        str_field: string
        int_field: 12
        level_dict:
            xyz: !Level
        level_array:
            - !Level
        level_direct: !Level
        more: any

    All fields are optional.
    """
    @classmethod
    def header(cls) -> str:
        return "base"

    @classmethod
    def schema(cls) -> Schema:
        return Schema(
            {
                Optional('$ref'): str,  # reference to other Base documents
                Optional('str_field'): str,
                Optional('int_field'): int,
                Optional('level_dict'): {
                    str: DocReference(Level)
                },
                Optional('level_array'): [DocReference(Level)],
                Optional('level_direct'): DocReference(Level),
                Optional('more'): lambda any: True
            }
        )

    def _load_subdocuments(self, lookup_paths: List[str]):
        if "level_dict" in self and self["level_dict"] != REMOVE:
            for key, doc in self["level_dict"].items():
                if doc != REMOVE:
                    self["level_dict"][key] = load_subdocument(doc, self, Level, lookup_paths)

        if "level_array" in self and self["level_array"] != REMOVE:
            new_level_array = []
            for doc in self["level_array"]:
                if isinstance(doc, dict):
                    new_level_array.append(load_subdocument(doc, self, Level, lookup_paths))
            self["level_array"] = new_level_array

        if "level_direct" in self and self["level_direct"] != REMOVE:
            self["level_direct"] = load_subdocument(self["level_direct"], self, Level, lookup_paths)
        return self

    @variable_helper
    def simple_helper(self):
        return "simple"


class Level(YamlConfigDocument):
    """
    Level test document. Example:

    level:
        name: str
        base_ref: !Base
        more: any

    All fields (except for name) are optional.
    """

    @classmethod
    def header(cls) -> str:
        return "level"

    @classmethod
    def schema(cls) -> Schema:
        return Schema(
            {
                Optional('$ref'): str,  # reference to other Level documents
                'name': str,
                Optional('base_ref'): DocReference(Base),
                Optional('more'): lambda any: True
            }
        )

    def _load_subdocuments(self, lookup_paths: List[str]):
        if "base_ref" in self:
            self["base_ref"] = load_subdocument(self["base_ref"], self, Base, lookup_paths)
        return self

    @variable_helper
    def level_helper(self):
        return "level"

