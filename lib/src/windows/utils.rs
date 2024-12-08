use std::string::FromUtf16Error;
use std::string::FromUtf8Error;

pub(crate) fn try_utf16_cstring<const N: usize>(
    value: &[u16; N],
) -> Result<String, FromUtf16Error> {
    let end_index = value
        .iter()
        .position(|&character| character == 0)
        .unwrap_or(0);
    String::from_utf16(&value[..end_index])
}

pub(crate) fn try_utf8_cstring<const N: usize>(value: &[u8; N]) -> Result<String, FromUtf8Error> {
    let end_index = value
        .iter()
        .position(|&character| character == 0)
        .unwrap_or(0);
    String::from_utf8(value[..end_index].to_vec())
}
