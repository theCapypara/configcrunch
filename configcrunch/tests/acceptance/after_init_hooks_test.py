import unittest
from copy import copy

from configcrunch.tests.fixtures.documents import Base
FIX = {"more": "data_modified_after_merge"}


class AfterInitTest(unittest.TestCase):
    def setUp(self) -> None:
        self.doc = Base.from_dict({
            "more": {
                "key_before": "value_before"
            }
        })

    def test_initialize_data_before_merge(self):
        def _initialize_data_before_merge(data):
            return copy(FIX)

        setattr(self.doc, '_initialize_data_before_merge', _initialize_data_before_merge)
        self.doc.resolve_and_merge_references([])
        self.assertEquals({"base": copy(FIX)}, self.doc.to_dict())

    def test_initialize_data_after_merge(self):
        def _initialize_data_after_merge(data):
            return copy(FIX)

        setattr(self.doc, '_initialize_data_after_merge', _initialize_data_after_merge)
        self.doc.resolve_and_merge_references([])
        self.assertEquals({"base": copy(FIX)}, self.doc.to_dict())

    def test_initialize_data_after_variables(self):
        def _initialize_data_after_variables(data):
            return copy(FIX)

        setattr(self.doc, '_initialize_data_after_variables', _initialize_data_after_variables)
        self.doc.process_vars()
        self.assertEquals({"base": copy(FIX)}, self.doc.to_dict())

    def test_initialize_data_after_freeze(self):
        def _initialize_data_after_freeze():
            self.doc["more"] = copy(FIX)

        setattr(self.doc, '_initialize_data_after_freeze', _initialize_data_after_freeze)
        self.doc.freeze()
        self.assertEquals({"more": copy(FIX)}, self.doc.items())
