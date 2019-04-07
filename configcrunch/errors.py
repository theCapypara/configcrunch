class ConfigcrunchError(Exception):
    pass


class ReferencedDocumentNotFound(ConfigcrunchError):
    pass


class CircularDependencyError(ConfigcrunchError):
    pass


class VariableProcessingError(ConfigcrunchError):
    pass


class InvalidDocumentError(ConfigcrunchError):
    pass


class InvalidHeaderError(InvalidDocumentError):
    pass


class InvalidRemoveError(InvalidDocumentError):
    pass
