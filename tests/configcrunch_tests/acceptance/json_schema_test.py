import json
import unittest

from configcrunch_tests.acceptance.testcases import ConfigcrunchTestCase
from configcrunch_tests.fixtures.documents import JsonRefLevel, NoRef, OptionalJsonRefs, RequiredJsonRefs


class JsonSchemaTest(ConfigcrunchTestCase):
    @classmethod
    def fixture_name(cls) -> str:
        return 'json_schema'

    @classmethod
    def fix_get_json(cls, path: str) -> dict:
        with open(cls.fix_get_path(path), 'r') as file:
            return json.load(file)

    def test_no_ref_json_schema(self):
        # Arrange
        expected_schema = self.fix_get_json('expected_no_ref_schema.json')
        # Act
        schema_result = NoRef.json_schema('no_ref_id')
        # Assert
        self.assertEqual(1, len(schema_result))
        self.assertIn('no_ref_id', schema_result)
        self.assertDictEqual(expected_schema, schema_result['no_ref_id'])

    def test_required_json_refs_json_schema(self):
        # Arrange
        expected_main_schema = self.fix_get_json('expected_required_json_refs_schema.json')
        expected_no_ref_schema = self.fix_get_json('expected_no_ref_schema.json')
        # Act
        schema_result = RequiredJsonRefs.json_schema('required_json_refs_id')
        # Assert
        self.assertEqual(2, len(schema_result))
        self.assertIn('required_json_refs_id', schema_result)
        self.assertIn('no_ref_id', schema_result)
        self.assertDictEqual(expected_main_schema, schema_result['required_json_refs_id'])
        self.assertDictEqual(expected_no_ref_schema, schema_result['no_ref_id'])

    def test_optional_json_refs_json_schema(self):
        # Arrange
        expected_main_schema = self.fix_get_json('expected_optional_json_refs_schema.json')
        expected_no_ref_schema = self.fix_get_json('expected_no_ref_schema.json')
        # Act
        schema_result = OptionalJsonRefs.json_schema('optional_json_refs_id')
        # Assert
        self.assertEqual(2, len(schema_result))
        self.assertIn('optional_json_refs_id', schema_result)
        self.assertIn('no_ref_id', schema_result)
        self.assertDictEqual(expected_main_schema, schema_result['optional_json_refs_id'])
        self.assertDictEqual(expected_no_ref_schema, schema_result['no_ref_id'])

    def test_json_ref_level_json_schema(self):
        # Arrange
        expected_main_schema = self.fix_get_json('expected_json_ref_level_schema.json')
        expected_level_schema = self.fix_get_json('expected_level_schema.json')
        # Act
        schema_result = JsonRefLevel.json_schema('json_ref_level_id')
        # Assert
        self.assertEqual(2, len(schema_result))
        self.assertIn('json_ref_level_id', schema_result)
        self.assertIn('level_ref_id', schema_result)
        self.assertDictEqual(expected_main_schema, schema_result['json_ref_level_id'])
        self.assertDictEqual(expected_level_schema, schema_result['level_ref_id'])
