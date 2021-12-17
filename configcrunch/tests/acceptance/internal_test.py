import unittest

from configcrunch.tests.fixtures.documents import Base


class InternalTest(unittest.TestCase):
    def setUp(self) -> None:
        self.doc = Base.from_dict({
            "more": {
                "key_before": "value_before"
            }
        })

    def test_internal_get_before_freeze(self):
        self.assertEquals({"key_before": "value_before"}, self.doc.internal_get("more"))
        self.assertEquals(None, self.doc.internal_get("doesnt_exist"))

    def test_internal_set_before_freeze(self):
        self.doc.internal_set("more", "new_more")
        self.doc.internal_set("new_key", "new_new_key")
        self.assertEquals("new_more", self.doc.internal_get("more"))
        self.assertEquals("new_new_key", self.doc.internal_get("new_key"))

    def test_internal_contains_before_freeze(self):
        self.assertTrue(self.doc.internal_contains("more"))
        self.assertFalse(self.doc.internal_contains("doesnt_exist"))

    def test_internal_delete_before_freeze(self):
        self.doc.internal_delete("more")
        self.assertFalse(self.doc.internal_contains("more"))
        # Deleting must work without an error
        self.doc.internal_delete("doesnt_exist")

    def test_internal_access_before_freeze(self):
        with self.doc.internal_access():
            self.doc["more"] = "new_more"
            self.doc["new_key"] = "new_new_key"
            self.doc.internal_set("internal_new_key", "new_internal_new_key")
        self.assertEquals("new_more", self.doc.internal_get("more"))
        self.assertEquals("new_new_key", self.doc.internal_get("new_key"))
        self.assertEquals("new_internal_new_key", self.doc.internal_get("internal_new_key"))

    def test_internal_get_after_freeze(self):
        self.doc.freeze()
        self.assertEquals({"key_before": "value_before"}, self.doc.internal_get("more"))
        self.assertEquals(None, self.doc.internal_get("doesnt_exist"))

    def test_internal_set_after_freeze(self):
        self.doc.freeze()
        self.doc.internal_set("more", "new_more")
        self.doc.internal_set("new_key", "new_new_key")
        self.assertEquals("new_more", self.doc.internal_get("more"))
        self.assertEquals("new_new_key", self.doc.internal_get("new_key"))

    def test_internal_contains_after_freeze(self):
        self.doc.freeze()
        self.assertTrue(self.doc.internal_contains("more"))
        self.assertFalse(self.doc.internal_contains("doesnt_exist"))

    def test_internal_delete_after_freeze(self):
        self.doc.freeze()
        self.doc.internal_delete("more")
        self.assertFalse(self.doc.internal_contains("more"))
        # Deleting must work without an error
        self.doc.internal_delete("doesnt_exist")

    def test_internal_access_after_freeze(self):
        self.doc.freeze()
        with self.doc.internal_access():
            self.doc["more"] = "new_more"
            self.doc["new_key"] = "new_new_key"
            self.doc.internal_set("internal_new_key", "new_internal_new_key")
        self.assertEquals("new_more", self.doc.internal_get("more"))
        self.assertEquals("new_new_key", self.doc.internal_get("new_key"))
        self.assertEquals("new_internal_new_key", self.doc.internal_get("internal_new_key"))
