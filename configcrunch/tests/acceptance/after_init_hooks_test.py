from configcrunch import load_multiple_yml
from configcrunch.tests.fixtures.documents import Base
from configcrunch.tests.acceptance.testcases import ConfigcrunchTestCase, deep_sort


class AfterInitHooks(ConfigcrunchTestCase):
    @classmethod
    def fixture_name(cls):
        return 'advanced_loader'

    def test_initialize_data_before_merge(self):
        self.fail("todo")

    def test_initialize_data_after_merge(self):
        self.fail("todo")

    def test_initialize_data_after_variables(self):
        self.fail("todo")

    def test_initialize_data_after_freeze(self):
        self.fail("todo")
