use prost::{DecodeError, Message};

use crate::services::ycchat::v1::models::PageToken;

pub fn get_page_token(page_token: String) -> Result<PageToken, DecodeError> {
    let decoded = super::base64_encoder::decode(page_token).unwrap();

    let buf = bytes::Bytes::from(decoded);

    PageToken::decode(buf)
}

pub trait PageTokenizer {
    fn generate_page_token(
        &self,
        page_size: i32,
        prev_page_token: Option<String>,
    ) -> Option<String>;
}

pub trait PageItem {
    fn get_item_id(&self) -> String;
}

impl<ITEM: PageItem> PageTokenizer for Vec<ITEM> {
    fn generate_page_token(
        &self,
        page_size: i32,
        prev_page_token: Option<String>,
    ) -> Option<String> {
        let offset_id = self.last().map(|item| item.get_item_id().to_string());

        let next_page_token = PageToken {
            page_size,
            order_by: None,
            offset_id,
            prev_page_token,
        };

        let mut pb_buf = vec![];
        let _ = next_page_token.encode(&mut pb_buf);

        let buf = super::base64_encoder::encode_string(pb_buf);

        Some(buf)

        // todo!("test")
    }
}
