Helper functions (Variable helpers)
-----------------------------------

As shown in the previous chapter, documents can contain variables and call
helper functions, so called "variable helpers". You can define your own helpers,
by adding methods to your document and adding the :func:`~configcrunch.variable_helper`
decorator to them.

Inside the variable helper method you can access all fields of your document, as well as all other
methods and variable helpers. All documents have a helper method called :func:`~configcrunch.YamlConfigDocument.parent`
that returns the parent document.

.. testcode:: main

    from schema import Schema, SchemaError, Optional
    from configcrunch import YamlConfigDocument, variable_helper

    class Example(YamlConfigDocument):
        @classmethod
        def header(cls) -> str:
            return "example"

        def schema(self) -> Schema:
            return Schema({
                'this': str,
                'int': int
            })

        @variable_helper
        def my_helper(self, param):
            return self['int'] + param

Example usage:

.. literalinclude:: fixtures/helpers.yml
   :language: yaml

This will result in the following document:

.. literalinclude:: fixtures/expected_results/helpers1.yml
   :language: yaml

.. doctest:: main
   :hide:

    >>> actual = Example.from_yaml("fixtures/helpers.yml")
    >>> actual.process_vars()
    Example(...)
    >>> expected = Example.from_yaml("fixtures/expected_results/helpers1.yml")
    >>> str(actual) == str(expected)
    True
