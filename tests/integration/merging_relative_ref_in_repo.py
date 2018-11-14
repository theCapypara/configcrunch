from tests.integration.testcases import ConfigcrunchTestCase


class MergingRelativeRefInRepo(ConfigcrunchTestCase):
    @classmethod
    def fixture_name(cls):
        return 'merging_relative_ref_in_repo'

    def test_one_repo(self):
        self.assertDocEqual(
            'expected_one_repo.yml',
            'base.yml',
            ['repo1']
        )

    def test_two_repos(self):
        self.assertDocEqual(
            'expected_two_repo.yml',
            'base.yml',
            ['repo1', 'repo2']
        )
