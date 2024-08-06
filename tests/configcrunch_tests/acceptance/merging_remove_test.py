from configcrunch_tests.acceptance.testcases import ConfigcrunchTestCase


class MergingRemove(ConfigcrunchTestCase):
    @classmethod
    def fixture_name(cls):
        return 'merging_remove'

    def test_dict_ref(self):
        self.assertDocEqualMerging(
            'expected.yml',
            'base.yml',
            ['repo']
        )

    def test_remove_subdoc(self):
        self.assertDocEqualMerging(
            'expected_subdoc.yml',
            'base_subdoc.yml',
            ['repo']
        )

    def test_remove_subdoc_multi_repo(self):
        self.assertDocEqualMerging(
            'expected_subdoc_multi_repo.yml',
            'base_subdoc_multi_repo.yml',
            ['repo', 'repo2']
        )

    def test_multi_level_list(self):
        self.assertDocEqualMerging(
            'multi_list_merge_expected.yml',
            'multi_list_merge_base.yml',
            ['repo']
        )
