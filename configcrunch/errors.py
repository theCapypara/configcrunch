class InvalidHeaderError(Exception):
    pass


class ReferencedDocumentNotFound(Exception):
    pass


class CircularDependencyError(Exception):
    pass


class VariableProcessingError(Exception):
    pass