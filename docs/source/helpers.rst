Helper functions (Variable helpers)
-----------------------------------

As shown in the previous chapter, documents can contain variables and call
helper functions, so called "variable helpers". You can define your own helpers,
by adding methods to your document and adding the :func:`~configcrunch.variable_helper`
decorator to them.

Inside the variable helper method you can access all fields of your document, as well as all other
methods and variable helpers. All documents have a helper method called :func:`~configcrunch.YamlConfigDocument.parent`
that returns the parent document.

.. testsetup:: main

    # A bit annoying, and might break without the Riptide/Docker setup :(
    import os
    if not os.path.exists("./fixtures"):
        os.chdir("/src/docs/source")

.. testcode:: main

    from schema import Schema, SchemaError, Optional
    from configcrunch import YamlConfigDocument, variable_helper

    class Example(YamlConfigDocument):
        @classmethod
        def header(cls) -> str:
            return "example"

        @classmethod
        def schema(cls) -> Schema:
            return Schema({
                'this': str,
                'int': int
            })

        @classmethod
        def subdocuments(cls):
            return []

        @variable_helper
        def my_helper(self, param):
            # Since variable helpers are called while the document isn't frozen yet,
            # you need to use the internal_... methods to get data from the document,
            # see the "Accessing Data" section.
            return self.internal_get('int') + param

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
    >>> actual.to_dict() == expected.to_dict()
    True
