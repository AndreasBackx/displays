#[derive(Debug, Clone, Copy, PartialEq, Eq, glib::ErrorDomain)]
#[error_domain(name = "AstalDisplaysError")]
pub enum AstalDisplaysError {
    Failed,
}

pub fn error_message(kind: AstalDisplaysError, message: &str) -> glib::Error {
    glib::Error::new(kind, message)
}
