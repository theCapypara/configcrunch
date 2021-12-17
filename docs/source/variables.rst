Variables and templating
------------------------

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
                Optional('list'): list
            })

        @classmethod
        def subdocuments(cls):
            return []

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
                ("direct", Example),
                ("map[]", Example)
            ]

Documents can contain template strings that are parsed using `Minijinja <https://github.com/mitsuhiko/minijinja>`_.

Variables are bound to documents. To use documents, include them in template strings:

.. literalinclude:: fixtures/variables.yml
   :language: yaml

To process variables, call :func:`~configcrunch.YamlConfigDocument.process_vars`.

.. doctest:: main

    >>> document = Example.from_yaml("fixtures/variables.yml")
    >>> print(document.internal_get('map')['key'])  # See the "Accessing Data" section
    {{ map.key2 }} <- all references are made from the root of the document

    >>> document.process_vars()
    Example(...)

    >>> document.freeze()
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

    >>> actual.to_dict() == expected.to_dict()
    True


.. warning::
    It IS supported to reference fields in templates that contain other templates.

    BUT it is NOT supported to reference fields with template
    strings when using the :func:`~configcrunch.YamlConfigDocument.parent` helper in any way.

Iterating
~~~~~~~~~
Configcrunch supports iteration over lists and over dicts (use ``.keys()``,``.values()`` or ``.items()``
depending on what you need to iterate over).

Value type interpretation
~~~~~~~~~~~~~~~~~~~~~~~~~
Configcrunch keeps the types of values as they are in the documents. The only expectation to this is
values that contain only variables and are parse-able as an integer. These are auto-converted to integers
for convenience. If you don't want that and need to interpret them as strings instead, use the ``str`` filter
as described below.

Filters
~~~~~~~
Minijinja supports filters, which are explained in more detail in the  `Minijinja documentation <https://docs.rs/minijinja/0.8.2/minijinja/filters/index.html>`_.

All built-in filters can be used. In addition the following filters exist:

- ``str`` (``{{ var|str }}``):
        (Only works if the template contains ONLY this one statement and nothing else!)
        Forces the value to be interpreted as a string, even if it could be auto-converted to an integer.

- ``substr_start`` (``{{ var|substr_start(X) }}``):
        Returns the first ``X`` characters of the string ``var``.

- ``startswith`` (``{{ var|startswith(string) }}``):
        Returns ``true`` if the string ``var`` starts with ``string``, else ``false``.
