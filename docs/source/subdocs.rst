Nesting documents
-----------------

Documents can contain sub-documents of the same or different types.

To allow the inclusion of sub-documents in your document,
add a :class:`~configcrunch.DocReference` to your schema at the position that you
expect a sub-document at.

To actually process sub-documents, you need to override the
:func:`~configcrunch.YamlConfigDocument.resolve_and_merge_references` method and
call :func:`~configcrunch.load_subdocument` at the positions that you expect a sub-document.

Example for a document class ``Parent`` that includes an ``Example`` document from the previous
chapters at either ``direct`` or in the map ``map`` as values:


.. testsetup:: main

    from schema import Schema, SchemaError, Optional
    from configcrunch import YamlConfigDocument

    class Example(YamlConfigDocument):
        @classmethod
        def header(cls) -> str:
            return "example"

        def schema(self) -> Schema:
            return Schema({
                'this': str,
                Optional('int'): int,
                Optional('map'): dict,
                Optional('list'): list
            })

.. testcode:: main

    from configcrunch import DocReference, load_subdocument

    class Parent(YamlConfigDocument):
        @classmethod
        def header(cls) -> str:
            return "parent"

        def schema(self) -> Schema:
            return Schema({
                'name': str,
                'direct': DocReference(Example),
                'map': {str: DocReference(Example)}
            })

        def resolve_and_merge_references(self, lookup_paths):
            super().resolve_and_merge_references(lookup_paths)

            # direct entry processing
            self["direct"] = load_subdocument(self["direct"], self, Example, lookup_paths)

            # map entry processing
            for key, doc in self["map"].items():
                self["map"][key] = load_subdocument(doc, self, Example, lookup_paths)

            return self

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
    ...   print(err)
    Expected an instance of Example while validating.

You can access sub-documents like other fields:

.. doctest:: main

    >>> print(document['direct'])
    Example({'this': 'is an example-type document'})
    >>> print(document['direct']['this'])
    is an example-type document