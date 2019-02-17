from configcrunch.tests.acceptance.testcases import ConfigcrunchTestCase


class MergingThreeRepos(ConfigcrunchTestCase):
    @classmethod
    def fixture_name(cls):
        return 'merging_three_repos'

    def test_no_subdoc(self):
        self.assertDocEqualMerging(
            'expected/no_subdoc.yml',
            'no_subdoc.yml',
            ['repo1', 'repo2', 'repo3']
        )

    def test_one_layer_subdoc(self):
        self.assertDocEqualMerging(
            'expected/one_level.yml',
            'one_level.yml',
            ['repo1', 'repo2', 'repo3']
        )

    def test_two_layers_subdoc(self):
        self.assertDocEqualMerging(
            'expected/two_level.yml',
            'two_level.yml',
            ['repo1', 'repo2', 'repo3']
        )
