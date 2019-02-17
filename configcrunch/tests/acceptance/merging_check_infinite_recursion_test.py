from configcrunch import CircularDependencyError
from configcrunch.tests.fixtures.documents import Base
from configcrunch.tests.acceptance.testcases import ConfigcrunchTestCase


class MergingCheckRefNotFound(ConfigcrunchTestCase):
    @classmethod
    def fixture_name(cls):
        return 'merging_check_infinite_recursion'

    def test_in_itself(self):
        base = Base.from_yaml(self.fix_get_path('in_itself.yml'))
        self.assertRaises(CircularDependencyError, base.resolve_and_merge_references, [self.fix_get_path('repo')])

    def test_via_child(self):
        base = Base.from_yaml(self.fix_get_path('via_child.yml'))
        self.assertRaises(CircularDependencyError, base.resolve_and_merge_references, [self.fix_get_path('repo')])
