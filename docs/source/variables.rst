Variables and templating
------------------------

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

    from configcrunch import DocReference, load_subdocument, REMOVE

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

        def _load_subdocuments(self, lookup_paths):
            # direct entry processing
            if self["direct"] != REMOVE:
                self["direct"] = load_subdocument(self["direct"], self, Example, lookup_paths)

            # map entry processing
            if self["map"] != REMOVE:
                for key, doc in self["map"].items():
                    if doc != REMOVE:
                        self["map"][key] = load_subdocument(doc, self, Example, lookup_paths)

            return self

Documents can contain template strings based on `Jinja2 <http://jinja.pocoo.org/docs/2.10/>`_.
Only basic referencing of variables and calling of functions is supported, although in theory most
of Jinja2 can be used.

Variables are bound to documents. To use documents, include them in template strings:

.. literalinclude:: fixtures/variables.yml
   :language: yaml

To process variables, call :func:`~configcrunch.YamlConfigDocument.process_vars`.

.. doctest:: main

    >>> document = Example.from_yaml("fixtures/variables.yml")
    >>> print(document['map']['key'])
    {{ map.key2 }} <- all references are made from the root of the document

    >>> document.process_vars()
    Example(...)

    >>> print(document['this'])
    12 is a number
    >>> print(document['map']['key2'])
    value 2
    >>> print(document['map']['key'])
    value 2 <- all references are made from the root of the document

You can also use variables with merged documents and sub-documents, but make sure to call :func:`~configcrunch.YamlConfigDocument.resolve_and_merge_references`
before :func:`~configcrunch.YamlConfigDocument.process_vars`!

Sub-documents can reference values in parent documents by using the :func:`~configcrunch.YamlConfigDocument.parent`
variable helper function. Variable helpers are further explained in the following chapter.

.. literalinclude:: fixtures/variables_parent.yml
   :language: yaml

.. doctest:: main

    >>> document = Parent.from_yaml("fixtures/variables_parent.yml")
    >>> document.resolve_and_merge_references([])
    Parent(...)
    >>> document.process_vars()
    Parent(...)

This will result in the following document:

.. literalinclude:: fixtures/expected_results/vars1.yml
   :language: yaml

.. doctest:: main
   :hide:

    >>> actual = document
    >>> expected = Parent.from_yaml("fixtures/expected_results/vars1.yml")
    >>> expected.resolve_and_merge_references([]) # doctest: +ELLIPSIS
    Parent(...)

    >>> str(actual) == str(expected)
    True


.. warning::
    It IS supported to reference fields in templates that contain other templates.

    BUT it is NOT supported to reference fields with template
    strings when using the :func:`~configcrunch.YamlConfigDocument.parent` helper in any way.
