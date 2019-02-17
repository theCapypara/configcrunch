from configcrunch.tests.acceptance.testcases import ConfigcrunchTestCase


class MergingEasyTest(ConfigcrunchTestCase):
    @classmethod
    def fixture_name(cls):
        return 'merging_easy'

    def test_same(self):
        self.assertDocEqualMerging(
            'easy.yml',
            'easy.yml',
            []
        )

    def test_different(self):
        doc = self.load_base('something_else.yml', [])
        expected_result = self.fix_get_yml('easy.yml')

        self.assertNotEqual(expected_result, doc.to_dict())
        self.assertValidDoc(doc)

    def test_different_direct(self):
        easy_doc = self.load_base('easy.yml', [])
        other_doc = self.load_base('something_else.yml', [])

        self.assertNotEqual(easy_doc.to_dict(), other_doc.to_dict())
        self.assertValidDoc(easy_doc)
        self.assertValidDoc(other_doc)
