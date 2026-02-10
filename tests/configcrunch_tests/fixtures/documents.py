"""
Classes that implement YamlConfigDocument and represent
YAML documents for the tests.
"""
from typing import List, Tuple, Type

from schema import Schema, Optional, Or, Regex

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
                Optional('str_field'): Or(str, None),
                Optional('int_field'): int,
                Optional('level_dict'): {
                    str: DocReference(Level)
                },
                Optional('level_array'): [DocReference(Level)],
                Optional('level_direct'): DocReference(Level),
                Optional('more'): lambda any: True
            },
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
            },
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


class NoRef(YamlConfigDocument):
    """
    Test document without any DocReferences. Example:

    noref:
        bool_field: true
        int_or_string_field: 12
        optional_regex_field: v4
        nested_object:
            nested_bool_field: false

    All fields except 'optional_regex_field' are required.
    """

    @classmethod
    def header(cls) -> str:
        return "noref"

    @classmethod
    def schema(cls) -> Schema:
        return Schema(
            {
                "bool_field": bool,
                "int_or_string_field": Or(int, str),
                Optional("optional_regex_field"): Regex(r"v\d+"),
                "nested_object": {
                    "nested_bool_field": bool,
                }
            },
            name="noref"
        )


class RequiredJsonRefs(YamlConfigDocument):
    """
    Test document with required DocReferences and some json schema ids. Example:

    required_json_refs:
        str_field: string
        required_ref_object: !NoRef
        list_ref_objects:
            - !NoRef
        no_id_ref_object: !Level

    All fields are required.
    """

    @classmethod
    def header(cls) -> str:
        return "required_json_refs"

    @classmethod
    def schema(cls) -> Schema:
        return Schema(
            {
                "str_field": str,
                "required_ref_object": DocReference(NoRef, "no_ref_id"),
                "nested_object": {
                    "nested_ref_object": DocReference(NoRef, "no_ref_id"),
                },
                "list_ref_objects": [DocReference(NoRef, "no_ref_id")],
                "no_id_ref_object": DocReference(Level),
            },
            name="required_json_refs",
        )


class OptionalJsonRefs(YamlConfigDocument):
    """
    Test document with optional DocReferences and some json schema ids. Example:

    optional_json_refs:
        str_field: string
        optional_ref_object: !NoRef
        nested_object:
            nested_ref_object: !NoRef
        list_ref_objects:
            - !NoRef
        no_id_ref_object: !Level

    All fields except 'str_field' and 'nested_object' are optional.
    """

    @classmethod
    def header(cls) -> str:
        return "optional_json_refs"

    @classmethod
    def schema(cls) -> Schema:
        return Schema(
            {
                "str_field": str,
                Optional("optional_ref_object"): DocReference(NoRef, "no_ref_id"),
                "nested_object": {
                    Optional("nested_ref_object"): DocReference(NoRef, "no_ref_id"),
                },
                Optional("list_ref_objects"): [DocReference(NoRef, "no_ref_id")],
                Optional("no_id_ref_object"): DocReference(Level),
            },
            name="optional_json_refs",
        )


class JsonRefLevel(YamlConfigDocument):
    """
    Test document with a DocReference to Level and a json schema id.
    Level doesn't contain any json schema ids. Example:

    json_ref_level:
        level_ref_object: !Level

    All fields are required.
    """

    @classmethod
    def header(cls) -> str:
        return "json_ref_level"

    @classmethod
    def schema(cls) -> Schema:
        return Schema(
            {
                "level_ref_object": DocReference(Level, "level_ref_id"),
            },
            name="json_ref_level",
        )
