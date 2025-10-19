from schema import SchemaError

from configcrunch_tests.acceptance.testcases import ConfigcrunchTestCase
from configcrunch_tests.fixtures.documents import Base


class NullValues(ConfigcrunchTestCase):
    @classmethod
    def fixture_name(cls):
        return 'null_values'

    def test_empty_value(self):
        doc = self.load_base('empty_value.yml', [])
        expected_result = self.fix_get_yml('empty_value.yml')

        self.assertValidDoc(doc)
        self.assertEqual(expected_result, doc.to_dict())
        self.assertEqual(None, doc.internal_get("str_field"))
        self.assertTrue(doc.validate())
        doc.freeze()
        self.assertEqual(None, doc["str_field"])

    def test_null_value(self):
        doc = self.load_base('null_value.yml', [])
        expected_result = self.fix_get_yml('null_value.yml')

        self.assertValidDoc(doc)
        self.assertEqual(expected_result, doc.to_dict())
        self.assertEqual(None, doc.internal_get("str_field"))
        self.assertTrue(doc.validate())
        doc.freeze()
        self.assertEqual(None, doc["str_field"])

    def test_tilde_value(self):
        doc = self.load_base('tilde_value.yml', [])
        expected_result = self.fix_get_yml('tilde_value.yml')

        self.assertValidDoc(doc)
        self.assertEqual(expected_result, doc.to_dict())
        self.assertEqual(None, doc.internal_get("str_field"))
        self.assertTrue(doc.validate())
        doc.freeze()
        self.assertEqual(None, doc["str_field"])

    def test_from_dict(self):
        doc = Base.from_dict({"str_field": None})

        self.assertValidDoc(doc)
        self.assertEqual(None, doc.internal_get("str_field"))
        self.assertTrue(doc.validate())
        doc.freeze()
        self.assertEqual(None, doc["str_field"])
