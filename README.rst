Configcrunch
============

|build| |docs|

.. |build| image:: http://jenkins.riptide.parakoopa.de:8080/buildStatus/icon?job=configcrunch%2Fmaster
    :target: http://jenkins.riptide.parakoopa.de:8080/blue/organizations/jenkins/configcrunch/activity
    :alt: Build Status

.. |docs| image:: https://readthedocs.org/projects/configcrunch/badge/?version=latest
    :target: https://configcrunch.readthedocs.io/en/latest/?badge=latest
    :alt: Documentation Status

Configcrunch is a Python library for reading YAML-based configuration files that aims to be simple
while also providing some very powerful features.

Configcrunch is comptible with Python 3.6 and up.

Install it via pip: ``pip install configcrunch``

Features:

- Read configuration files from YAML files.
- Define various types of configuration files, that can be validated via a schema.
- Types of configuration files are defined as seperate Python classes.
- Documents can be configured to contain sub-documents of any type.
- Documents can contain `Jinja2 <http://jinja.pocoo.org/docs/2.10/>`_ based variables that can
  reference any other field inside the same or parent document.
- The classes that represent your document types can contain methods that can be used
  inside the configuration files.
- Documents can reference documents from other files. Configcrunch will merge them together.
  You decide where referenced documents are looked up.
- Configuration objects can also be created without YAML files, by using default Python dicts.
- All features are optional.

Used by:

- `Riptide <https://github.com/Parakoopa/riptide-lib>`_
- (Your project here! Open an issue.)

By default Configcrunch uses `schema <https://pypi.org/project/schema/>`_ to validate schemas.
But you can also use your own validation logic!

Example
-------

This is an example that uses most of the features described above, using two document types.

.. code-block:: yaml

    # doc1.yml - Type: one
    one:
        name: Document
        number: 1
        sub:
            # Sub-document of type "two"
            $ref: /doc2
            two_field: "{{ parent().method() }}"


.. code-block:: yaml

    # <lookup path>/doc2.yml - Type: two
    two:
        name: Doc 2
        number: 2
        two_field: This is overridden


.. code-block:: python

    # classes.py
    from schema import Schema, Optional

    from configcrunch import YamlConfigDocument, DocReference, load_subdocument, variable_helper


    class One(YamlConfigDocument):
        @classmethod
        def header(cls) -> str:
            return "one"

        def schema(self) -> Schema:
            return Schema(
                {
                    Optional('$ref'): str,  # reference to other One documents
                    'name': str,
                    'number': int,
                    Optional('sub'): DocReference(Two)
                }
            )

        def resolve_and_merge_references(self, lookup_paths):
            super().resolve_and_merge_references(lookup_paths)
            if "sub" in self:
                self["sub"] = load_subdocument(self["sub"], self, Two, lookup_paths)
            return self

        @variable_helper
        def method(self):
            return "I will return something"


    class Two(YamlConfigDocument):
        @classmethod
        def header(cls) -> str:
            return "two"

        def schema(self) -> Schema:
            return Schema(
                {
                    Optional('$ref'): str,  # reference to other Two documents
                    'name': str,
                    'number': int,
                    'two_field': str
                }
            )


The document "one.yml" can then be read via Python:

    >>> import yaml
    >>> from classes import One
    >>> doc = One.from_yaml('./doc1.yml')
    >>> doc.resolve_and_merge_references(['<lookup path>'])
    >>> doc.process_vars()
    >>> print(yaml.dump(doc.to_dict(), default_flow_style=False))
    one:
      name: Document
      number: 1
      sub:
        name: Doc 2
        number: 2
        two_field: I will return something


Tests
-----

Inside the ``configcrunch.tests`` package are acceptance tests. Unit tests are WIP.

To run the tests, see ``run_tests.sh``.

Documentation
-------------

The complete documentation can be found at `Read the Docs <https://configcrunch.readthedocs.io/en/latest/>`_ (or in the docs directory).