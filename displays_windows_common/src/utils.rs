use std::string::FromUtf16Error;

pub fn try_utf16_cstring<const N: usize>(value: &[u16; N]) -> Result<String, FromUtf16Error> {
    let end_index = value
        .iter()
        .position(|&character| character == 0)
        .unwrap_or(0);
    String::from_utf16(&value[..end_index])
}

pub fn get_gdi_device_id(path: &str) -> Option<u32> {
    path.chars()
        .last()
        .and_then(|c| c.to_digit(10))
        .map(|digit| digit)
}
