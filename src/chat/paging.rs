use std::marker::PhantomData;

use ulid::Ulid;

use crate::redis::page_token::{PageToken, PageTokenKey};

pub struct PagingManager<ITEM, P>
where
    P: Pager<ITEM>,
{
    pager: P,
    _phantom: PhantomData<(ITEM, P)>,
}

pub struct PagingResult<T> {
    pub list: Vec<T>,
    pub total_size: u64,
    pub next_page_token: Option<String>,
    pub prev_page_token: Option<String>,
}

impl<ITEM, P> PagingManager<ITEM, P>
where
    P: Pager<ITEM>,
{
    pub fn new(pager: P) -> Self {
        PagingManager {
            pager,
            _phantom: PhantomData,
        }
    }

    pub fn paging(
        &self,
        id: String,
        page_token_id: Option<String>,
        page_size: u64,
    ) -> PagingResult<ITEM> {
        let page_token_key = page_token_id
            .as_ref()
            .map(|page_token| self.pager.generate_page_token_key(&id, page_token));

        let page_token = if let Some(page_token_key) = page_token_key {
            self.pager.get_page_token(page_token_key)
        } else {
            None
        };

        let (start, end) = if let Some(token) = &page_token {
            let offset_id = token.offset_id.clone();

            let start = self.pager.get_start_index(&id, &offset_id);

            let start = start;

            let end = start + isize::try_from(token.size).unwrap_or(0);

            (start, end)
        } else {
            (0, isize::try_from(page_size).unwrap_or(0))
        };

        let end = 0.max(end - 1);

        let list = self.pager.get_list(&id, start, end);

        let total_size = self.pager.get_total_size(&id);

        let is_have_next = u64::try_from(end).unwrap() < total_size;

        let (next_page_token_id, prev_page_token_id) = match &page_token {
            Some(page_token) => (
                page_token.next_page_token.clone(),
                page_token.prev_page_token.clone(),
            ),
            None => (None, None),
        };

        let next_page_token = if is_have_next {
            let first_page_token = if page_token_id.is_none() {
                let page_token = PageToken::new(None, page_size, None);

                let page_token_key = self
                    .pager
                    .generate_page_token_key(&id, &page_token.id.clone().unwrap());

                self.pager.set_page_token(page_token_key, &page_token);

                // self.set_chat_room_page_token(&user_id, page_token.clone());

                Some(page_token)
            } else {
                None
            };

            let offset_id = self.pager.get_offset_id(&list.last());

            let next_page_token = self.generate_next_page_token(
                &next_page_token_id,
                if page_token.is_none() {
                    first_page_token.map(|item| item.id.unwrap())
                } else {
                    page_token_id
                },
                offset_id,
                page_size,
                None,
            );

            if let Some(next_page_token_id) = &next_page_token.id {
                let page_token_key = self.pager.generate_page_token_key(&id, next_page_token_id);

                self.pager.set_page_token(page_token_key, &next_page_token);
            }

            next_page_token.id
        } else {
            None
        };

        PagingResult {
            list,
            total_size,
            next_page_token,
            prev_page_token: prev_page_token_id,
        }
    }

    fn generate_next_page_token(
        &self,
        id: &Option<String>,
        prev_token: Option<String>,
        offset_id: Option<String>,
        size: u64,
        order_by: Option<String>,
    ) -> PageToken {
        let mut next_page_token = PageToken::new(offset_id, size, order_by);

        next_page_token.set_next_page_token(Ulid::new().to_string());

        if let Some(id) = id {
            next_page_token.set_id(id.clone());
        }

        if let Some(prev_token) = prev_token {
            next_page_token.set_prev_page_token(prev_token);
        }

        next_page_token
    }
}

pub trait Pager<ITEM> {
    fn get_total_size(&self, parent_id: &String) -> u64;
    fn get_list(&self, parent_id: &String, start: isize, end: isize) -> Vec<ITEM>;
    fn get_offset_id(&self, item: &Option<&ITEM>) -> Option<String>;
    fn get_start_index(&self, parent_id: &String, offset_id: &Option<String>) -> isize;

    fn set_page_token(&self, page_token_key: PageTokenKey, page_token: &PageToken);
    fn get_page_token(&self, page_token_key: PageTokenKey) -> Option<PageToken>;
    fn generate_new_page_token_key(&self, user_id: &str) -> PageTokenKey;
    fn generate_page_token_key(&self, id: &str, page_token_id: &str) -> PageTokenKey;
}
