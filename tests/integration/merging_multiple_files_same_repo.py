from tests.integration.testcases import ConfigcrunchTestCase


class MergingMultipleFilesSameRepo(ConfigcrunchTestCase):
    @classmethod
    def fixture_name(cls):
        return 'merging_multiple_files_same_repo'

    def test_same(self):
        self.assertDocEqual(
            'expected.yml',
            'base.yml',
            ['repo']
        )
