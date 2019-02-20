from configcrunch import InvalidHeaderError, InvalidDocumentError
from configcrunch.tests.fixtures.documents import Base
from configcrunch.tests.acceptance.testcases import ConfigcrunchTestCase


class MergingCheckInvalidSubdoc(ConfigcrunchTestCase):
    @classmethod
    def fixture_name(cls):
        return 'merging_check_invalid_subdoc'

    def test_invalid(self):
        self.assertRaises(InvalidHeaderError, Base.from_yaml, self.fix_get_path('base_invalid.yml'))

    def test_invalid_in_ref(self):
        base = Base.from_yaml(self.fix_get_path('base_valid.yml'))
        self.assertRaises(InvalidHeaderError, base.resolve_and_merge_references, [self.fix_get_path('repo')])

    def test_invalid_empty(self):
        self.assertRaises(InvalidDocumentError, Base.from_yaml, self.fix_get_path('base_empty.yml'))

    def test_invalid_level_empty(self):
        base = Base.from_yaml(self.fix_get_path('base_valid_level_empty.yml'))
        self.assertRaises(InvalidDocumentError, base.resolve_and_merge_references, [self.fix_get_path('repo')])
