from configcrunch.tests.acceptance.testcases import ConfigcrunchTestCase


class Variables(ConfigcrunchTestCase):
    @classmethod
    def fixture_name(cls):
        return 'variables'

    def test_none(self):
        self.assertDocEqualVariables(
            'none.yml',
            'none.yml'
        )

    def test_one_level(self):
        self.assertDocEqualVariables(
            'expected/one_level.yml',
            'one_level.yml'
        )

    def test_helpers(self):
        self.assertDocEqualVariables(
            'expected/helper_calls.yml',
            'helper_calls.yml'
        )

    def test_accessing_child_vars(self):
        self.assertDocEqualVariables(
            'expected/accessing_child_vars.yml',
            'accessing_child_vars.yml'
        )

    def test_complex(self):
        self.assertDocEqualVariables(
            'expected/complex.yml',
            'complex.yml'
        )

    def test_not_working(self):
        """
        When calling parent() and accessing a field on the parent with variables in it,
        the variables are not correctly resolved*. This also happens when trying to access a field of a
        subdocument of the parent that contains a variable and hasn't been processed yet.

        Therefor such things are not currently not allowed. If this becomes possible in the future
        feel free to rename and change this test.

        *: To be exact: The variable value is copied over to the subdocument as-is and processed there in it's context.
        """
        self.assertDocEqualVariables(
            'expected/not_working.yml',
            'not_working.yml'
        )
