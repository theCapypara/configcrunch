Configcrunch
============

Configcrunch is a Python library for reading YAML-based configuration files that aims to be simple
while also providing some very powerful features.

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

Documentation
-------------

The complete documentation can be found at `Read the Docs <https://configcrunch.readthedocs.io/en/latest/>`_ (or in the docs directory).