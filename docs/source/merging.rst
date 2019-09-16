Document merging and lookup paths
---------------------------------

The basics
~~~~~~~~~~

Documents (and sub-documents!) can reference other files. These files will be loaded first and
then the documents are merged on top of them. This allows you to overwrite and merge configuration files.

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

The following examples will use the  ``Parent`` and ``Example`` document types from the previous chapters.

Let's say you have the following configuration file (type ``Parent``):

.. literalinclude:: fixtures/parent_with_ref.yml
   :language: yaml

This document contains a ``$ref`` entry. When calling the :func:`~configcrunch.YamlConfigDocument.resolve_and_merge_references`
on a :class:`~configcrunch.YamlConfigDocument` Configcrunch looks for files with this relative
path in fhe array you provide it. Let's say you have the following file (note the path):

.. literalinclude:: fixtures/repo/referenced-document.yml
   :language: yaml

You can tell Configcrunch to look in the directory ``fixtures/repo`` while trying to load ``parent_with_ref.yml``.
Configcrunch will notice the ``$ref`` key, look for ``referenced-document.yml`` and merge the two files:

.. doctest:: main

    >>> document = Parent.from_yaml("fixtures/parent_with_ref.yml")
    >>> document.resolve_and_merge_references(["fixtures/repo"]) # doctest: +ELLIPSIS
    Parent(...)

    >>> print(document['name'])
    overwritten
    >>> print(document['direct']['this'])
    foo
    >>> print(document['direct']['int'])
    1234

The list passed to :func:`~configcrunch.YamlConfigDocument.resolve_and_merge_references` is a list of lookup paths.

As you can see the resulting document is a combination of the two documents. All values in ``referenced-document.yml``
were replaced with values from ``parent_with_refs.yml``. This also spans sub-documents.

The resulting document is (overwrites from ``parent_with_ref.yml`` are emphasized):

.. literalinclude:: fixtures/expected_results/merge1.yml
   :language: yaml
   :emphasize-lines: 2,5,9-10

.. doctest:: main
   :hide:

    >>> actual = document
    >>> expected = Parent.from_yaml("fixtures/expected_results/merge1.yml")
    >>> expected.resolve_and_merge_references([]) # doctest: +ELLIPSIS
    Parent(...)

    >>> str(actual) == str(expected)
    True

Chaining references
~~~~~~~~~~~~~~~~~~~
You can chain $ref-Entries. If a document is $ref'erenced this document can contain a $ref-entry
as well. It's $ref reference will be processed first.

Documents that are referenced can reference other documents relative to their position within
the reference paths.

Example document in lookup paths:

.. literalinclude:: fixtures/repo/referenced-document-with-reference.yml
   :language: yaml

Example document that loads this document:

.. literalinclude:: fixtures/parent_with_ref_chain.yml
   :language: yaml


Example after merge:

.. literalinclude:: fixtures/expected_results/merge3.yml
   :language: yaml

.. doctest:: main
   :hide:

    >>> actual = Parent.from_yaml("fixtures/parent_with_ref_chain.yml")
    >>> actual.resolve_and_merge_references(["fixtures/repo"]) # doctest: +ELLIPSIS
    Parent(...)
    >>> expected = Parent.from_yaml("fixtures/expected_results/merge3.yml")
    >>> expected.resolve_and_merge_references([]) # doctest: +ELLIPSIS
    Parent(...)

    >>> str(actual) == str(expected)
    True


References in sub-documents
~~~~~~~~~~~~~~~~~~~~~~~~~~~
Sub documents can also contain ``$ref`` entries. They will be merged as expected. For this to work
you only need to set up sub-documents as explained in the last chapter.

You place these documents to include also in the lookup paths:

.. literalinclude:: fixtures/repo/examples/referenced.yml
   :language: yaml

You can then use it with ``$ref``-entries:

.. literalinclude:: fixtures/parent_with_ref_and_sub.yml
   :language: yaml


This will result in the following document being merged:

.. literalinclude:: fixtures/expected_results/merge1.yml
   :language: yaml

.. doctest:: main
   :hide:

    >>> actual = Parent.from_yaml("fixtures/parent_with_ref_and_sub.yml")
    >>> actual.resolve_and_merge_references(["fixtures/repo"]) # doctest: +ELLIPSIS
    Parent(...)
    >>> expected = Parent.from_yaml("fixtures/expected_results/merge2.yml")
    >>> expected.resolve_and_merge_references([]) # doctest: +ELLIPSIS
    Parent(...)

    >>> str(actual) == str(expected)
    True

.. warning::
    Sub-documents are loaded after parent documents are merged, so $ref-Entries in sub-documents
    can be overwritten if they are present in referenced parent documents. They are NOT chained
    in this case.

    As an example, imagine ``parent_with_ref.yml`` and ``referenced-document.yml`` would both
    contain a ``$ref``-entry for the sub-document under direct. Only the ``$ref``-entry from ``parent_with_ref.yml``
    will be processed.

Multiple lookup paths
~~~~~~~~~~~~~~~~~~~~~

You can provide multiple lookup paths.

If a document is found in multiple lookup paths, the documents will be processed in the
order of the entries in the lookup paths list. They will then be merged as explained under
"Chaining references".

This allows a lookup path to "override" another. The first lookup path is the base lookup path
and the documents in the other lookup paths can extend and change definitions in the lookup
paths that come before them.

Removing entries
~~~~~~~~~~~~~~~~

During merging, a document can remove something from a referenced document with the special
``$remove`` keyword.

To remove string entries from lists, add an entry to the list which has the original value and prefix
it with "$remove::". Only removing strings from lists is supported.

Example document:

.. literalinclude:: fixtures/parent_with_ref_and_delete.yml
   :language: yaml
   :emphasize-lines: 8,10

Merge result:

.. literalinclude:: fixtures/expected_results/merge4.yml
   :language: yaml

.. doctest:: main
   :hide:

    >>> actual = Parent.from_yaml("fixtures/parent_with_ref_and_delete.yml")
    >>> actual.resolve_and_merge_references(["fixtures/repo"]) # doctest: +ELLIPSIS
    Parent(...)
    >>> expected = Parent.from_yaml("fixtures/expected_results/merge4.yml")
    >>> expected.resolve_and_merge_references([]) # doctest: +ELLIPSIS
    Parent(...)

    >>> str(actual) == str(expected)
    True