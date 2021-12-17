use pyo3::create_exception;
create_exception!(_main, ConfigcrunchError, pyo3::exceptions::PyException);
create_exception!(_main, ReferencedDocumentNotFound, ConfigcrunchError);
create_exception!(_main, CircularDependencyError, ConfigcrunchError);
create_exception!(_main, VariableProcessingError, ConfigcrunchError);
create_exception!(_main, InvalidDocumentError, ConfigcrunchError);
create_exception!(_main, InvalidHeaderError, InvalidDocumentError);
create_exception!(_main, InvalidRemoveError, InvalidDocumentError);

pyo3::import_exception!(schema, SchemaError);
