from configcrunch import ReferencedDocumentNotFound
from tests.fixtures.documents import Base
from tests.acceptance.testcases import ConfigcrunchTestCase


class MergingCheckRefNotFound(ConfigcrunchTestCase):
    @classmethod
    def fixture_name(cls):
        return 'merging_check_ref_not_found'

    def test_invalid_ref(self):
        base = Base.from_yaml(self.fix_get_path('base.yml'))
        self.assertRaises(ReferencedDocumentNotFound, base.resolve_and_merge_references, [])
