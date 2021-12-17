Migrating to Version 1.x
------------------------

Version 1 of Configcrunch was fully rewritten and is in some crucial aspects incompatible
with older versions.

``__init__`` is no longer called
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
The ``__init__`` method on classes subclassing ``YamlConfigDocument`` is no longer called
due to various issues with integration related to it in the Rust codebase. As an alternative,
the method ``_initialize_data_before_merge`` might be useful.

$name is added to all documents in dicts
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
Configcrunch adds an entry with the key ``$name`` to all sub-documents that are contained
in dicts. The value is the key in the dictionary they are in. Be sure to update your schemas
to allow this.

Removed and moved functions
~~~~~~~~~~~~~~~~~~~~~~~~~~~
If you previously loaded in functions or classes from anywhere but the main ``configcrunch``
module, this will now no longer work. ``load_multiple_yml`` has also been moved to ``configcrunch``.
Additionally the function ``load_subdocument`` is no longer available, see below.

New sub-documents logic
~~~~~~~~~~~~~~~~~~~~~~~
The way sub-documents are defined and processed has been simplified. Instead of manually having
to manually load them in ``_load_subdocuments`` you now have to define the classmethod ``subdocuments``,
which returns a list of tuples that define the locations sub-documents are expected at. See the manual page
on `sub-documents </subdocs.html>`_ for more information.

.. code-block:: python

    # Old method:
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

    # New method:
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

Variable processing
~~~~~~~~~~~~~~~~~~~
Variable and template processing has been replaced with `Minijinja <https://github.com/mitsuhiko/minijinja>`_,
which is for the most part compatible with Jinja2. See the manual page on `variables </variables.html>`_ for
more information.

Freezing documents and internal_* methods
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
Documents can no longer be accessed like a dict, and the ``.doc`` property is no longer available, unless
the document is frozen using the ``freeze()`` method first. Instead you can use the ``internal_*`` methods.

See the manual page on `accessing data </accessing_data.html>`_ for more information.
