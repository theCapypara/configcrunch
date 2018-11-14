# Constants
REF = "$ref"
REMOVE = "$remove"

# Public classes and functions
from .abstract import YamlConfigDocument, DocReference
from .merger import resolve_and_merge, load_subdocument
from .errors import *
