Accessing Data
--------------

As you have seen in the previous chapters of this documentation,
documents need to be frozen in order to be directly accessed like a dictionary.

However there are also ways to retrieve and manipulate the data without the document
being frozen. This chapter will show you all ways to interact with and manipulate your documents.

``internal_*`` methods
~~~~~~~~~~~~~~~~~~~~~~
Using the methods
:func:`~configcrunch.YamlConfigDocument.internal_get`,
:func:`~configcrunch.YamlConfigDocument.internal_set`,
:func:`~configcrunch.YamlConfigDocument.internal_delete` and
:func:`~configcrunch.YamlConfigDocument.internal_contains`
you can get, set and delete values at any time, or check if the
document contains them. Please note, that if the document is not
frozen yet, the ``internal_get`` returns a COPY of the values inside,
which means that none of the changes done to nested data structures
(except for nested documents themself) will be reflected, so you need
to use ``internal_set`` to write these changes back to the document.

If the document is already frozen, these methods act like the dict-like
access or the ``.doc`` property access.

The method :func:`~configcrunch.YamlConfigDocument.internal_access` returns
a context manager which can be used to temporarily simulate a freeze on
the document. You can then access the document like a dict, or use ``.doc``
to manipulate the document. After the context, the document will re-synchronize
the internal state with all the changes you made.
Please note that this can be potentially relatively expensive with big documents.

Dict-like access, ``.doc`` property, ``.items()``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
If the document was frozen with the :func:`~configcrunch.YamlConfigDocument.freeze`
method, the data gets copied into the ``.doc`` property, which is an ordinary
``dict`` which can also be directly accessed via item access on the document itself.

Please note that frozen documents can not be further merged or processed, the methods
``resolve_and_merge_references``, ``process_vars`` and ``validate`` will raise exceptions
on frozen documents. You can not un-freeze documents. If you need to temporarily
have dict-like access to the document, use ``internal_access`` instead (see above).

The method ``items()`` returns the ``.doc`` property too.

Copying the data via ``.to_dict()``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
Calling :func:`~configcrunch.YamlConfigDocument.to_dict` at any time on the document
will return a dict containing the header as a key, and the document's internal dict
as a value. All sub-documents are also converted to dicts.

Hooks
~~~~~

You can also implement these hook methods on your classes, to manipulate the data during specific
events of the lifecycle of the document:

.. method:: _initialize_data_before_merge(self, data: dict) -> dict
  :abstractmethod:

    May be used to initialize the document by adding / changing data.

    Called before doing anything else in :func:`~configcrunch.YamlConfigDocument.resolve_and_merge_references`.
    Use this for setting default values.

    The internal data is passed as an argument and can be mutated.
    The changed data MUST be returned again.


.. method:: _initialize_data_after_merge(self, data: dict) -> dict
  :abstractmethod:

    May be used to initialize the document by adding / changing data.

    Called after :func:`~configcrunch.YamlConfigDocument.resolve_and_merge_references`.
    Use this for setting default values.

    The internal data is passed as an argument and can be mutated.
    The changed data MUST be returned again.


.. method:: _initialize_data_after_variables(self, data: dict) -> dict
  :abstractmethod:

    May be used to initialize the document by adding / changing data.

    Called after :func:`~configcrunch.YamlConfigDocument.process_vars`.

    Use this for setting internal values based on processed values in the document.
    The internal data is passed as an argument and can be mutated.
    The changed data MUST be returned again.


.. method:: _initialize_data_after_freeze(self)
  :abstractmethod:

    May be used to initialize the document by adding / changing data.

    Called after :func:`~configcrunch.YamlConfigDocument.freeze`.

    Use this for setting internal values based on processed values in the document.
    You can access the data using the ``self.doc`` property or by getting it from self (``self[...]``).
