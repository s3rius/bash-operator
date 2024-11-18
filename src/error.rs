pub type BOResult<T> = Result<T, BOError>;

#[derive(Debug, thiserror::Error)]
pub enum BOError {
    #[error("KubeError: {0}")]
    KubeError(#[from] kube::Error),
    #[error("SerdeError: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("IOError: {0}")]
    IOError(#[from] std::io::Error),
    #[error("ParseGroupVersionError: {0}")]
    ParseGroupVersionError(#[from] kube::core::gvk::ParseGroupVersionError),
    #[error("Cannot parse manifest file!")]
    ParseManifestError,
    #[error("SerdeYamlError: {0}")]
    SerdeYamlError(#[from] serde_yaml::Error),
    #[error("Incorrect status code has been returned: {0}.")]
    ErrorStatusCode(i32),
    #[error("Failed to conver to inner object: {0}")]
    IntoInnerError(#[from] std::io::IntoInnerError<std::io::BufWriter<tempfile::NamedTempFile>>),
}
