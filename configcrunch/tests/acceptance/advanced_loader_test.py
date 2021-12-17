from configcrunch import load_multiple_yml
from configcrunch.tests.fixtures.documents import Base
from configcrunch.tests.acceptance.testcases import ConfigcrunchTestCase, deep_sort


class AdvancedLoader(ConfigcrunchTestCase):
    @classmethod
    def fixture_name(cls):
        return 'advanced_loader'

    def test_none(self):
        doc = load_multiple_yml(
            Base, self.fix_get_path('deep.yml'), self.fix_get_path('middle.yml'), self.fix_get_path('top.yml')
        )
        expected_result = self.fix_get_yml('expected.yml')

        self.assertDictEqual(deep_sort(expected_result), deep_sort(doc.to_dict()))
        self.assertValidDoc(doc)
