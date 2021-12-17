import unittest
from configcrunch._main import _test__subdoc_specs

FIXTURE = {
    "level1": {
        "level2": {
            "list2": ["hello", "world", "how", "are", "you"],
            "dict2": {"lev2_1": "lev2_1_val", "lev2_2": "lev2_2_val"}
        },
        "list1": ["hi", "level 1"],
        "dict1": {
            "lev1_1": "lev1_1_val",
            "lev1_2": "lev1_2_val",
            "lev1_3": "lev1_3_val"
        }
    },
    "direct_dict": {"replace": "me"},
    "direct": "hi",
    "list": ["list1", "list2"]
}


class SubdocSpecsTest(unittest.TestCase):
    def test_subdoc_specs(self):
        self.assertEqual(
            ({
                "level1": {
                    "level2": {
                        "list2": ["hello", "world", "how", "are", "you"],
                        "dict2": {"lev2_1": "lev2_1_val", "lev2_2": "lev2_2_val"}
                    },
                    "list1": ["hi", "level 1"],
                    "dict1": {
                        "lev1_1": "lev1_1_val",
                        "lev1_2": "lev1_2_val",
                        "lev1_3": "lev1_3_val"
                    }
                },
                "direct_dict": "REPLACED",
                "direct": "hi",
                "list": ["list1", "list2"]
            }, type),
            _test__subdoc_specs("direct_dict", type, FIXTURE, "REPLACED")
        )
        self.assertEqual(
            ({
                "level1": {
                    "level2": {
                        "list2": ["hello", "world", "how", "are", "you"],
                        "dict2": {"lev2_1": "lev2_1_val", "lev2_2": "lev2_2_val"}
                    },
                    "list1": ["hi", "level 1"],
                    "dict1": {
                        "lev1_1": "lev1_1_val",
                        "lev1_2": "lev1_2_val",
                        "lev1_3": "lev1_3_val"
                    }
                },
                "direct_dict": {"replace": "me"},
                "direct": "hi",
                "list": "REPLACED"
            }, type),
            _test__subdoc_specs("list", type, FIXTURE, "REPLACED")
        )
        self.assertEqual(
            ({
                "level1": {
                    "level2": {
                        "list2": ["hello", "world", "how", "are", "you"],
                        "dict2": {"lev2_1": "lev2_1_val", "lev2_2": "lev2_2_val"}
                    },
                    "list1": ["hi", "level 1"],
                    "dict1": {
                        "lev1_1": "lev1_1_val",
                        "lev1_2": "lev1_2_val",
                        "lev1_3": "lev1_3_val"
                    }
                },
                "direct_dict": {"replace": "me"},
                "direct": "hi",
                "list": ["REPLACED", "REPLACED"]
            }, type),
            _test__subdoc_specs("list[]", type, FIXTURE, "REPLACED")
        )
        self.assertEqual(
            ({
                "level1": {
                    "level2": {
                        "list2": ["hello", "world", "how", "are", "you"],
                        "dict2": {"lev2_1": "lev2_1_val", "lev2_2": "lev2_2_val"}
                    },
                    "list1": ["hi", "level 1"],
                    "dict1": {
                        "lev1_1": "lev1_1_val",
                        "lev1_2": "lev1_2_val",
                        "lev1_3": "lev1_3_val"
                    }
                },
                "direct_dict": {"replace": "REPLACED"},
                "direct": "hi",
                "list": ["list1", "list2"]
            }, type),
            _test__subdoc_specs("direct_dict[]", type, FIXTURE, "REPLACED")
        )
        self.assertEqual(
            ({
                "level1": {
                    "level2": {
                        "list2": ["hello", "world", "how", "are", "you"],
                        "dict2": {"lev2_1": "lev2_1_val", "lev2_2": "lev2_2_val"}
                    },
                    "list1": ["REPLACED", "REPLACED"],
                    "dict1": {
                        "lev1_1": "lev1_1_val",
                        "lev1_2": "lev1_2_val",
                        "lev1_3": "lev1_3_val"
                    }
                },
                "direct_dict": {"replace": "me"},
                "direct": "hi",
                "list": ["list1", "list2"]
            }, type),
            _test__subdoc_specs("level1/list1[]", type, FIXTURE, "REPLACED")
        )
        self.assertEqual(
            ({
                "level1": {
                    "level2": {
                        "list2": ["hello", "world", "how", "are", "you"],
                        "dict2": "REPLACED"
                    },
                    "list1": ["hi", "level 1"],
                    "dict1": {
                        "lev1_1": "lev1_1_val",
                        "lev1_2": "lev1_2_val",
                        "lev1_3": "lev1_3_val"
                    }
                },
                "direct_dict": {"replace": "me"},
                "direct": "hi",
                "list": ["list1", "list2"]
            }, type),
            _test__subdoc_specs("level1/level2/dict2", type, FIXTURE, "REPLACED")
        )
        self.assertEqual(
            ({
                "level1": {
                    "level2": {
                        "list2": ["hello", "world", "how", "are", "you"],
                        "dict2": {"lev2_1": "REPLACED", "lev2_2": "REPLACED"}
                    },
                    "list1": ["hi", "level 1"],
                    "dict1": {
                        "lev1_1": "lev1_1_val",
                        "lev1_2": "lev1_2_val",
                        "lev1_3": "lev1_3_val"
                    }
                },
                "direct_dict": {"replace": "me"},
                "direct": "hi",
                "list": ["list1", "list2"]
            }, type),
            _test__subdoc_specs("level1/level2/dict2[]", type, FIXTURE, "REPLACED")
        )
