Schema and validating
---------------------

Configcrunch can be used to validate documents. Configcrunch validates
documents against a schema. Documents are not validated on load, a special
method has to be called instead.

To validate documents, `schema <https://github.com/keleshev/schema>`_ is used.
For more examples and a full documentation please have a look at the
schema documentation.

Let's say we have the following document:

.. literalinclude:: fixtures/simple_example.yml
   :language: yaml

To add a schema, we define the ``schema`` method of the document class. In this
example all fields shown are optional, except for ``this``:

.. testcode:: main

    from schema import Schema, SchemaError, Optional
    from configcrunch import YamlConfigDocument

    class Example(YamlConfigDocument):
        @classmethod
        def header(cls) -> str:
            return "example"

        @classmethod
        def schema(cls) -> Schema:
            return Schema({
                'this': str,
                Optional('int'): int,
                Optional('map'): dict,
                Optional('list'): list
            })

You can then use the validate method on documents. On valid documents it returns ``True``
and doesn't raise an error:

.. doctest:: main

    >>> document = Example.from_yaml('fixtures/simple_example.yml')
    >>> print(document.validate())
    True

If you have an invalid document, like the one below, you will get a Schema error
detailing the issue.

.. literalinclude:: fixtures/invalid.yml
   :language: yaml

.. doctest:: main

    >>> document = Example.from_yaml('fixtures/invalid.yml')
    >>> try:
    ...   document.validate()
    ... except SchemaError as err:
    ...   print(err)
    123 should be instance of 'str'
