from configcrunch import load_multiple_yml
from configcrunch.tests.fixtures.documents import Base
from configcrunch.tests.acceptance.testcases import ConfigcrunchTestCase, deep_sort


class InternalTest(ConfigcrunchTestCase):
    @classmethod
    def fixture_name(cls):
        return 'internal'

    def test_internal_get(self):
        self.fail("todo")

    def test_internal_set(self):
        self.fail("todo")

    def test_internal_contains(self):
        self.fail("todo")

    def test_internal_delete(self):
        self.fail("todo")

    def test_internal_access(self):
        self.fail("todo")
