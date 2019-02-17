from configcrunch.tests.acceptance.testcases import ConfigcrunchTestCase


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
