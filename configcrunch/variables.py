from jinja2 import Environment
from typing import TYPE_CHECKING, Callable
from configcrunch.errors import VariableProcessingError

from configcrunch.interface import IYamlConfigDocument

if TYPE_CHECKING:
    from configcrunch.abstract import YamlConfigDocument

jinja2env = Environment()

something_changed = False


class DocumentTraverser:
    def __init__(self):
        self.something_changed = False

    def run_callback(self, callback: Callable, input_node, *args):
        self.something_changed = False
        return self.__traverse(callback, input_node, *args)

    def __traverse(self, callback: Callable, input_node, *args):
        if isinstance(input_node, dict):
            for key, value in input_node.items():
                input_node[key] = self.__traverse(callback, value, *args)
            return input_node

        elif isinstance(input_node, list):
            new_node = []
            for value in input_node:
                new_node.append(self.__traverse(callback, value, *args))
            input_node.clear()
            input_node += new_node
            return input_node

        else:
            return callback(self, input_node, *args)


def apply_variable_resolution(input_str: str, document: 'YamlConfigDocument'):
    # TODO doc
    template = jinja2env.from_string(input_str)

    for helper in document.bound_helpers:
        template.globals[helper.__name__] = helper

    return template.render(document.doc)


def __process_variables_for_subdoc(traverse: DocumentTraverser, input_node: any) -> any:
    if isinstance(input_node, IYamlConfigDocument):
        return process_variables(input_node)
    else:
        return input_node


def __process_variables_current_doc(traverse: DocumentTraverser, input_node, document: 'YamlConfigDocument'):
    """
    Recursive prcoess variables
    The input node is changed in place immediately for dict entries and after processing
    the entire list for list entries.
    :return: Merge result of step.
    """
    if isinstance(input_node, str):
        try:
            new_value = apply_variable_resolution(input_node, document)
        except Exception:
            raise VariableProcessingError("Error processing a variable for document. "
                                          "Original value was %s, document class is %s. "
                                          "Document path: %s (None means root document)."
                                          % (input_node, document.__class__.__name__, document.path))
        if new_value != input_node:
            traverse.something_changed = True
        return new_value
    else:
        return input_node


def process_variables(ycd: 'YamlConfigDocument'):
    # TODO: Currently the algorithm isn't very smart. It just runs over the
    # TODO: document, replacing variables, until no replacements have been done.
    traverse = DocumentTraverser()
    # First process all sub documents variables
    traverse.run_callback(__process_variables_for_subdoc, ycd.doc)
    # And now our variables
    still_has_variables = True
    while still_has_variables:
        traverse.run_callback(__process_variables_current_doc, ycd.doc, ycd)
        still_has_variables = traverse.something_changed
    return ycd


def process_variables_for(ycd: 'YamlConfigDocument', target: str):
    return apply_variable_resolution(target, ycd)

# http://jinja.pocoo.org/docs/2.10/api/
# http://jinja.pocoo.org/docs/2.10/api/#jinja2.runtime.Context
# http://jinja.pocoo.org/docs/2.10/sandbox/#jinja2.sandbox.SandboxedEnvironment
# http://jinja.pocoo.org/docs/2.10/extensions/
# https://stackoverflow.com/a/47291097
