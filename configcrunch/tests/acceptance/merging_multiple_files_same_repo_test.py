from configcrunch.tests.acceptance.testcases import ConfigcrunchTestCase


class MergingMultipleFilesSameRepo(ConfigcrunchTestCase):
    @classmethod
    def fixture_name(cls):
        return 'merging_multiple_files_same_repo'

    def test_same(self):
        self.assertDocEqualMerging(
            'expected.yml',
            'base.yml',
            ['repo']
        )
