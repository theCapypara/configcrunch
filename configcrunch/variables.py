from jinja2 import Environment, Undefined
from typing import TYPE_CHECKING
from configcrunch.errors import VariableProcessingError

from configcrunch.interface import IYamlConfigDocument

if TYPE_CHECKING:
    from configcrunch.abstract import YamlConfigDocument

jinja2env = Environment(undefined=Undefined)

something_changed = False

def _process_variables__recursion_subdocuments(input_node: any) -> any:

    # TODO In Methode auslagern mit callback
    if isinstance(input_node, dict):
        for key, value in input_node.items():
            input_node[key] = _process_variables__recursion_subdocuments(value)
        return input_node

    elif isinstance(input_node, list):
        new_node = []
        for value in input_node:
            new_node.append(_process_variables__recursion_subdocuments(value))
        input_node.clear()
        input_node += new_node
        return input_node

    if isinstance(input_node, IYamlConfigDocument):
        return process_variables(input_node)

    else:
        return input_node

def _process_variables__recursion(document: 'YamlConfigDocument', input_node: any) -> any:
    """
    Recursive prcoess variables
    The input node is changed in place immediately for dict entries and after processing
    the entire list for list entries.
    :return: Merge result of step.
    """
    # TODO: global is a no no.
    global something_changed
    if isinstance(input_node, dict):
        for key, value in input_node.items():
            input_node[key] = _process_variables__recursion(document, value)
        return input_node

    elif isinstance(input_node, list):
        new_node = []
        for value in input_node:
            new_node.append(_process_variables__recursion(document, value))
        input_node.clear()
        input_node += new_node
        return input_node

    elif isinstance(input_node, str):
        # TODO doc
        template = jinja2env.from_string(input_node)\

        for helper in document.bound_helpers:
            template.globals[helper.__name__] = helper

        try:
            new_value = template.render(document.doc)
        except Exception:
            raise VariableProcessingError("Error processing a variable for document. "
                                          "Original value was %s, document class is %s. "
                                          "Document path: %s (None means root document)."
                                          % (input_node, document.__class__.__name__, document.path))
        # Try to convert to number if it is one, and silently fail if not possible, which will keep
        # new_value being a string
        try:
            new_value = float(new_value)
        except ValueError:
            pass
        if new_value != input_node:
            something_changed = True
        return new_value

    else:
        return input_node


def process_variables(ycd: 'YamlConfigDocument'):
    # TODO: Currently the algorithm isn't very smart. It just runs over the
    # TODO: document, replacing variables, until no replacements have been done.

    # TODO: global is a no no.
    global something_changed
    # First process all sub documents variables
    _process_variables__recursion_subdocuments(ycd.doc)
    # And now our variables
    still_has_variables = True
    while still_has_variables:
        _process_variables__recursion(ycd, ycd.doc)
        still_has_variables = something_changed
        something_changed = False
    return ycd
