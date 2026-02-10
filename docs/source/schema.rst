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

.. testsetup:: main

    # A bit annoying, and might break without the Riptide/Docker setup :(
    import os
    if not os.path.exists("./fixtures"):
        os.chdir("/src/docs/source")

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

        @classmethod
        def subdocuments(cls):
            return []

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
    Key 'this' error:
    123 should be instance of 'str'

JSON Schema generation
~~~~~~~~~~~~~~~~~~~~~~

You can generate Draft-07 JSON Schemas for documents
by invoking the ``json_schema`` method.

If you are overriding ``schema`` for your document
and return a non-schema object it'll need to be an instance of
``dict`` or ``list`` and provide the following properties and methods:

- ``name: str|None``: The name of the schema.

- ``description: str|None``: The description of the schema.

- ``ignore_extra_keys: bool``: Whether the schema allows additional keys
  which aren't explictly defined.

- ``schema: schema.Schema|dict|list``: The inner schema, this should usually
  return a ``dict``.

- ``json_schema(json_schema_id: str) -> dict``: The method generating
  the JSON schema, based on the values of the properties above.

If your custom schema class is a child class of ``schema.Schema``,
it also needs to provide a writable ``_schema`` attribute,
which is usually the backing field of the ``schema`` property.
