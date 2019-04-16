import collections
import os
import unittest
from abc import abstractmethod, ABC

import yaml
from schema import SchemaError
from configcrunch.tests.fixtures.documents import Base


unittest.util._MAX_LENGTH = 2000


class ConfigcrunchTestCase(unittest.TestCase, ABC):

    def setUp(self):
        self.maxDiff = None

    @classmethod
    @abstractmethod
    def fixture_name(cls) -> str:
        pass

    @classmethod
    def __fixture_path(cls):
        return os.path.abspath(os.path.join(os.path.dirname(__file__), os.pardir, 'fixtures', cls.fixture_name()))

    @classmethod
    def fix_get_path(cls, path):
        return os.path.join(cls.__fixture_path(), path)

    @classmethod
    def fix_get_yml(cls, path):
        with open(cls.fix_get_path(path), 'r') as stream:
            return yaml.safe_load(stream)

    def load_base(self, path, lookup_paths):
        base = Base.from_yaml(self.fix_get_path(path))
        abs_lookup_paths = []
        for path in lookup_paths:
            abs_lookup_paths.append(self.fix_get_path(path))
        base.resolve_and_merge_references(abs_lookup_paths)
        return base

    def assertDocEqualMerging(self, expected_yml_file, input_yml_file, repo_folder_names):
        doc = self.load_base(input_yml_file, repo_folder_names)
        expected_result = self.fix_get_yml(expected_yml_file)

        self.assertDictEqual(deep_sort(expected_result), deep_sort(doc.to_dict()))
        self.assertValidDoc(doc)

    def assertDocEqualVariables(self, expected_yml_file, input_yml_file):
        doc = self.load_base(input_yml_file, [])
        doc.process_vars()
        expected_result = self.fix_get_yml(expected_yml_file)

        self.assertDictEqual(deep_sort(expected_result), deep_sort(doc.to_dict()))
        self.assertValidDoc(doc)

    def assertValidDoc(self, doc):
        try:
            self.assertTrue(doc.validate())
        except SchemaError:
            self.fail("The document is not valid")


""" Helper functions to guarantee list orders follow"""


def sortby(x):
    """Return integer hash for all non-numbers (if possible, otherwise inf). To make everything sortable"""
    if x.__class__.__lt__ != x.__lt__:
        if isinstance(x, collections.Hashable):
            return hash(x)
        return float('inf')
    return x


def deep_sort(obj):
    """Recursively sort list or nested lists"""
    if isinstance(obj, dict):
        _sorted = {}
        for key in sorted(obj, key=sortby):
            _sorted[key] = deep_sort(obj[key])

    elif isinstance(obj, list):
        new_list = []
        for val in obj:
            new_list.append(deep_sort(val))
        _sorted = sorted(new_list, key=sortby)

    else:
        _sorted = obj

    return _sorted
