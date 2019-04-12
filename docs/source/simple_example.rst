Getting started (simple example)
--------------------------------

To get started with Configcrunch you need to create a subclass of :class:`~configcrunch.YamlConfigDocument`.

This class will represent one type of document and can be used to load a YAML file based on it's schema.
You need to specify a header and a schema for the document.
In this simple example we'll use the header ``example`` and a schema that validates all documents.

.. testcode:: main

    from schema import Schema
    from configcrunch import YamlConfigDocument

    class Example(YamlConfigDocument):
        @classmethod
        def header(cls) -> str:
            return "example"

        def schema(self) -> Schema:
            return Schema(any)

This class can then be used to load all YAML documents with the header example:

.. include:: fixtures/simple_example.yml
   :code: yaml

To load the document, use your created class:

.. testcode:: main

    document = Example.from_yaml('fixtures/simple_example.yml')

The ``document`` object can then be used like a Python dict to get and retrieve values
in the document. You can also use the ``document.doc`` field to directly access the
dictionary that represents the document.

.. doctest:: main

    >>> print(document["this"])
    document can contain anything
    >>> print(document["list"][0])
    1
    >>> print(document.doc["this"])
    document can contain anything
    >>> print(document["this"])
    document can contain anything

To get a full dict representation of the document call  :func:`~configcrunch.YamlConfigDocument.to_dict`.
The dict will also contain the document header. This can be used with packages like pyyaml, to convert the document back into a YAML file.

.. doctest:: main

    >>> print(document.to_dict()["example"]["map"]["key"])
    value