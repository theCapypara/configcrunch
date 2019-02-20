from configcrunch import ReferencedDocumentNotFound
from configcrunch.tests.fixtures.documents import Base
from configcrunch.tests.acceptance.testcases import ConfigcrunchTestCase


class MergingCheckRefNotFound(ConfigcrunchTestCase):
    @classmethod
    def fixture_name(cls):
        return 'merging_check_ref_not_found'

    def test_invalid_ref(self):
        base = Base.from_yaml(self.fix_get_path('base.yml'))
        self.assertRaises(ReferencedDocumentNotFound, base.resolve_and_merge_references, [])

    def test_invalid_ref_relative(self):
        base = Base.from_yaml(self.fix_get_path('base_invalid_relative.yml'))
        self.assertRaises(ReferencedDocumentNotFound, base.resolve_and_merge_references, [])

    def test_invalid_ref_relative_parent(self):
        base = Base.from_yaml(self.fix_get_path('base_invalid_relative_parent.yml'))
        self.assertRaises(ReferencedDocumentNotFound, base.resolve_and_merge_references, [])

    def test_invalid_ref_relative_parent_two_layers(self):
        base = Base.from_yaml(self.fix_get_path('base_invalid_relative_parent_two_layers.yml'))
        self.assertRaises(ReferencedDocumentNotFound, base.resolve_and_merge_references, [self.fix_get_path('repo')])

