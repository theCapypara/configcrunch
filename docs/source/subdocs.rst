Nesting documents
-----------------

Documents can contain sub-documents of the same or different types.

To allow the inclusion of sub-documents in your document,
add a :class:`~configcrunch.DocReference` to your schema at the position that you
expect a sub-document at.

To actually process sub-documents, you need to specify, where they are located with the
return value of the :func:`~configcrunch.YamlConfigDocument.subdocuments` method.

Example for a document class ``Parent`` that includes an ``Example`` document from the previous
chapters at either ``direct`` or in the map ``map`` as values:


.. testsetup:: main

    # A bit annoying, and might break without the Riptide/Docker setup :(
    import os
    if not os.path.exists("./fixtures"):
        os.chdir("/src/docs/source")

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
                Optional('list'): list,
                # If this document is used as a sub-document in a dict, Configrunch will add an
                # entry to it with the key "$name" that contains the key in the dict.
                Optional('$name'): str,
            })

        @classmethod
        def subdocuments(cls):
            return []

.. testcode:: main

    from configcrunch import DocReference

    class Parent(YamlConfigDocument):
        @classmethod
        def header(cls) -> str:
            return "parent"

        @classmethod
        def schema(cls) -> Schema:
            return Schema({
                'name': str,
                'direct': DocReference(Example),
                'map': {str: DocReference(Example)}
            })

        @classmethod
        def subdocuments(cls):
            return [
                # direct entry processing
                ("direct", Example),
                # dict entry processing (also works for lists)
                ("map[]", Example),
                # If you have more complex documents, you can reference sub fields, by specifying
                # the path to it, seperated by /.
                # ("level0/level1", Example),
                # or ("level0/level1[]", Example),
            ]

The following document would be a valid document for ``Parent``:

.. literalinclude:: fixtures/parent.yml
   :language: yaml

To load and process this document, load the document like an ordinary document. After that call
:func:`~configcrunch.YamlConfigDocument.resolve_and_merge_references` on it. The parameter
is not relevant in this case (leave it as an empty list) and will be explained in the next chapter.

.. testcode:: main

    document = Parent.from_yaml("fixtures/parent.yml")
    document.resolve_and_merge_references([])

You can then validate the document. The sub-documents are validated against their schemas.
Validating the document before calling this function will return in a SchemaError.

.. doctest:: main

    >>> document.validate()
    True

    >>> document2 = Parent.from_yaml('fixtures/parent.yml')
    >>> # No calling of resolve_and_merge_references
    >>> try:
    ...   document2.validate()
    ... except SchemaError as err:
    ...   print(err)  # doctest: +IGNORE_RESULT - (order of validation errors is non-deterministic)
    Key 'map' error:
    Key 'one' error:
    Expected an instance of 'Example' while validating, got 'dict': {'int': 3, 'this': 'is also an example-type document'}

You can access sub-documents like other fields. Calling freeze on the parent document will also
freeze all sub-documents.

.. doctest:: main

    >>> document.freeze()
    >>> print(document['direct'])
    Example({'this': 'is an example-type document'})
    >>> print(document['direct']['this'])
    is an example-type document
    >>> print(document['map']['one']['$name'])  # This will be added to all sub-documents in dicts.
    one
