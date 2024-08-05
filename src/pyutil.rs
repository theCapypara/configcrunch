use pyo3::Python;

/// Like clone, but while holding the Python GIL.
pub(crate) trait ClonePyRef {
    fn clone_pyref(&self, py: Python) -> Self;
}
