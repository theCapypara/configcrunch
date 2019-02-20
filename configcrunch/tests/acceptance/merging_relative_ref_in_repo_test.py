from configcrunch.tests.acceptance.testcases import ConfigcrunchTestCase


class MergingRelativeRefInRepo(ConfigcrunchTestCase):
    @classmethod
    def fixture_name(cls):
        return 'merging_relative_ref_in_repo'

    def test_one_repo(self):
        self.assertDocEqualMerging(
            'expected_one_repo.yml',
            'base.yml',
            ['repo1']
        )

    def test_parent_repo(self):
        self.assertDocEqualMerging(
            'expected_one_repo.yml',
            'base_parent_repo.yml',
            ['repo_parent_directory']
        )

    def test_two_repos(self):
        self.assertDocEqualMerging(
            'expected_two_repo.yml',
            'base.yml',
            ['repo1', 'repo2']
        )
