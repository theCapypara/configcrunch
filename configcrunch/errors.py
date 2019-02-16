class ConfigcrunchError(Exception):
    pass


class InvalidHeaderError(ConfigcrunchError):
    pass


class ReferencedDocumentNotFound(ConfigcrunchError):
    pass


class CircularDependencyError(ConfigcrunchError):
    pass


class VariableProcessingError(ConfigcrunchError):
    pass
