"""
Classes that implement YamlConfigDocument and represent
YAML documents for the tests.
"""
from typing import List

from schema import Schema, Optional

from configcrunch import YamlConfigDocument, DocReference, load_subdocument
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

    def schema(self) -> Schema:
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

    def resolve_and_merge_references(self, lookup_paths: List[str]):
        super().resolve_and_merge_references(lookup_paths)
        if "level_dict" in self:
            for key, doc in self["level_dict"].items():
                self["level_dict"][key] = load_subdocument(doc, self, Level, lookup_paths)

        if "level_array" in self:
            new_level_array = []
            for doc in self["level_array"]:
                new_level_array.append(load_subdocument(doc, self, Level, lookup_paths))
            self["level_array"] = new_level_array

        if "level_direct" in self:
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

    def schema(self) -> Schema:
        return Schema(
            {
                Optional('$ref'): str,  # reference to other Level documents
                'name': str,
                Optional('base_ref'): DocReference(Base),
                Optional('more'): lambda any: True
            }
        )

    def resolve_and_merge_references(self, lookup_paths: List[str]):
        super().resolve_and_merge_references(lookup_paths)
        if "base_ref" in self:
            self["base_ref"] = load_subdocument(self["base_ref"], self, Base, lookup_paths)
        return self

    @variable_helper
    def level_helper(self):
        return "level"


"""
    Test cases:
    - Easy (one doc)                                            | merging_easy
    - One ref in main doc, one source in repo, nothing to merge | merging_one_ref
        - ... with something to merge                           
        - ... with a second repo to merge                       
    - No ref in main doc, but subdocuments                      | merging_subdoc
        - dict, with and without $ref in subdoc                                        
            - with $ref in main doc too                         
        - list, with and without $ref in subdoc                                      
            - with $ref in main doc too                        
        - direct, with $ref in subdoc                                     
            - with $ref in main doc too                         
    - One ref in main doc, one source in repo, ref. doc relative| merging_relative_ref_in_repo
        - with two repos                                        
    - $remove:                                                  | merging_remove
        - scalar
        - list
        - dict
        - YamlConfigDocument
        - $ref
        - as element in list
        - as key in dict
    - test with three repositories, no subdocuments             | merging_three_repos
        - one layer subdocument
        - two layer subdocument with relative imports in those
    - infinite recursion check                                  | merging_check_infinite_recursion
    - $ref not found check                                      | merging_check_ref_not_found
    - one ref with .yml and .yaml check                         | merging_multiple_files_same_repo
    - invalid subdocument type/header check                     | merging_check_invalid_subdoc
    - negative validation: one document                         | negative_validation
        - one subdocument
        - one $ref with two repos
    
    later:
    - git test
    - tests for variable resolution...
"""
