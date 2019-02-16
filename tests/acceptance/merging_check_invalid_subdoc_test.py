from configcrunch import InvalidHeaderError
from tests.fixtures.documents import Base
from tests.acceptance.testcases import ConfigcrunchTestCase


class MergingCheckInvalidSubdoc(ConfigcrunchTestCase):
    @classmethod
    def fixture_name(cls):
        return 'merging_check_invalid_subdoc'

    def test_invalid(self):
        self.assertRaises(InvalidHeaderError, Base.from_yaml, self.fix_get_path('base_invalid.yml'))

    def test_invalid_in_ref(self):
        base = Base.from_yaml(self.fix_get_path('base_valid.yml'))
        self.assertRaises(InvalidHeaderError, base.resolve_and_merge_references, [self.fix_get_path('repo')])
