use thiserror::Error;

#[derive(Error, Debug)]
pub enum WindowsError {
    #[error("Windows error occurred")]
    Known {
        #[from]
        source: windows::core::Error,
    },
    #[error("invalid utf-16 provided for {origin}: {data:?}")]
    InvalidUtf16 { data: Vec<u16>, origin: String },
    #[error("Windows error occured with not much extra info: {message}")]
    Other { message: String },
}
