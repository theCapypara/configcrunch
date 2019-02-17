from configcrunch.tests.acceptance.testcases import ConfigcrunchTestCase


class MergingOneRef(ConfigcrunchTestCase):
    @classmethod
    def fixture_name(cls):
        return 'merging_one_ref'

    def test_one_repo_nothing_to_merge(self):
        self.assertDocEqualMerging(
            'expected.yml',
            'base.yml',
            ['repo1']
        )

    def test_one_repo_something_to_merge(self):
        self.assertDocEqualMerging(
            'expected_with_something_to_merge.yml',
            'base_with_something_to_merge.yml',
            ['repo1']
        )

    def test_two_repos_something_to_merge(self):
        self.assertDocEqualMerging(
            'expected_with_something_to_merge_two_repos.yml',
            'base_with_something_to_merge.yml',
            ['repo1', 'repo2']
        )
