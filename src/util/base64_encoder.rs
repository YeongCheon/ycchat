use base64::{
    alphabet,
    engine::{self, general_purpose},
    DecodeError, Engine as _,
};

pub fn encode_string(data: Vec<u8>) -> String {
    let mut buf = String::new();

    engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD)
        .encode_string(data, &mut buf);

    buf
}

pub fn decode(data: String) -> Result<Vec<u8>, DecodeError> {
    engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD).decode(data)
}
