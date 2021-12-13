from configcrunch import load_multiple_yml
from configcrunch.tests.fixtures.documents import Base
from configcrunch.tests.acceptance.testcases import ConfigcrunchTestCase, deep_sort


class InternalTest(ConfigcrunchTestCase):
    @classmethod
    def fixture_name(cls):
        return 'internal'

    def test_internal_get_before_freeze(self):
        self.fail("todo")

    def test_internal_set_before_freeze(self):
        self.fail("todo")

    def test_internal_contains_before_freeze(self):
        self.fail("todo")

    def test_internal_delete_before_freeze(self):
        self.fail("todo")

    def test_internal_access_before_freeze(self):
        self.fail("todo")

    def test_internal_get_after_freeze(self):
        self.fail("todo")

    def test_internal_set_after_freeze(self):
        self.fail("todo")

    def test_internal_contains_after_freeze(self):
        self.fail("todo")

    def test_internal_delete_after_freeze(self):
        self.fail("todo")

    def test_internal_access_after_freeze(self):
        self.fail("todo")
