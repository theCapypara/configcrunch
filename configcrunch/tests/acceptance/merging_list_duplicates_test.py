from configcrunch.tests.acceptance.testcases import ConfigcrunchTestCase


class MergingListDuplicates(ConfigcrunchTestCase):
    @classmethod
    def fixture_name(cls):
        return 'merging_list_duplicates'

    def test_lists(self):
        self.assertDocEqualMerging(
            'expected.yml',
            'base.yml',
            ['repo']
        )
