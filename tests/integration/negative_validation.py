from schema import SchemaError

from configcrunch import ReferencedDocumentNotFound
from tests.fixtures.documents import Base
from tests.integration.testcases import ConfigcrunchTestCase


class NegativeValidation(ConfigcrunchTestCase):
    @classmethod
    def fixture_name(cls):
        return 'negative_validation'

    def test_invalid_main(self):
        doc = self.load_base('negative.yml', [])
        self.assertRaises(SchemaError, doc.validate)

    def test_invalid_subdoc(self):
        doc = self.load_base('negative_subdocument.yml', [])
        self.assertRaises(SchemaError, doc.validate)
