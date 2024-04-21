use std::sync::Arc;

use reqwest::Client;
use serde::Serialize;

mod errors;
mod models;

#[derive(Clone)]
pub struct Billine {
    secret_key: String,
    client: Arc<Client>,
}

impl Billine {
    pub fn new(secret_key: String) -> Self {
        Self {
            secret_key,
            client: Arc::new(Client::new()),
        }
    }

    pub fn signed_request<T: Serialize>(data: T) {}
}
