#[derive(Debug, Clone, Copy, PartialEq, Eq, glib::ErrorDomain)]
#[error_domain(name = "DisplaysAstalError")]
pub enum DisplaysAstalError {
    Failed,
}

pub fn error_message(kind: DisplaysAstalError, message: &str) -> glib::Error {
    glib::Error::new(kind, message)
}
