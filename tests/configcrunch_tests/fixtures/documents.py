"""
Classes that implement YamlConfigDocument and represent
YAML documents for the tests.
"""
from typing import List, Tuple, Type

from schema import Schema, Optional

from configcrunch import YamlConfigDocument, DocReference, REMOVE, variable_helper


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

    @classmethod
    def subdocuments(cls) -> List[Tuple[str, Type[YamlConfigDocument]]]:
        return [
            ("level_dict[]", Level),
            ("level_array[]", Level),
            ("level_direct", Level),
        ]

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
                Optional('$name'): str,  # key if in a dict
                'name': str,
                Optional('base_ref'): DocReference(Base),
                Optional('more'): lambda any: True
            }
        )

    @classmethod
    def subdocuments(cls) -> List[Tuple[str, Type[YamlConfigDocument]]]:
        return [
            ("base_ref", Base),
        ]

    @variable_helper
    def level_helper(self):
        return "level"

    @variable_helper
    def level_helper_taking_param(self, param: str):
        return f"level_param: {param}"
