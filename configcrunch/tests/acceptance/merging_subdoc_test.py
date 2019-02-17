from configcrunch.tests.acceptance.testcases import ConfigcrunchTestCase


class MergingSubdoc(ConfigcrunchTestCase):
    @classmethod
    def fixture_name(cls):
        return 'merging_subdoc'

    def test_dict_ref(self):
        self.assertDocEqualMerging(
            'expected/dict_ref.yml',
            'dict_ref.yml',
            ['repo']
        )

    def test_dict_ref_with_maindoc_ref(self):
        self.assertDocEqualMerging(
            'expected/dict_ref_with_maindoc_ref.yml',
            'dict_ref_with_maindoc_ref.yml',
            ['repo']
        )

    def test_direct_ref(self):
        self.assertDocEqualMerging(
            'expected/direct_ref.yml',
            'direct_ref.yml',
            ['repo']
        )

    def test_direct_ref_with_maindoc_ref(self):
        self.assertDocEqualMerging(
            'expected/direct_ref_with_maindoc_ref.yml',
            'direct_ref_with_maindoc_ref.yml',
            ['repo']
        )

    def test_list_ref(self):
        self.assertDocEqualMerging(
            'expected/list_ref.yml',
            'list_ref.yml',
            ['repo']
        )

    def test_list_ref_with_maindoc_ref(self):
        self.assertDocEqualMerging(
            'expected/list_ref_with_maindoc_ref.yml',
            'list_ref_with_maindoc_ref.yml',
            ['repo']
        )
