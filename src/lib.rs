use std::{collections::HashMap, sync::Arc};

use reqwest::Client;
use serde::Serialize;

mod errors;
mod models;

pub use errors::*;
pub use models::*;

#[derive(Clone)]
pub struct Billine {
    secret_key: String,
    base_url: String,
    client: Arc<Client>,
}

impl Billine {
    pub fn new(secret_key: String, base_url: String) -> Self {
        Self {
            secret_key,
            client: Arc::new(Client::new()),
            base_url,
        }
    }

    pub async fn signed_get_request<T: Serialize>(
        &self,
        path: &str,
        data: T,
    ) -> Result<String, errors::Error> {
        let mut hashmap_serialized: HashMap<String, serde_json::Value> = serde_json::from_value(
            serde_json::to_value(&data).expect("Serialization to value failed"),
        )
        .unwrap();

        let signature = sha256_signature(&data, &self.secret_key);

        hashmap_serialized.insert("sign".into(), serde_json::Value::String(signature));

        let complete_url = format!("{}{}", self.base_url, path);

        let serialized_request = serde_json::to_string(&hashmap_serialized).unwrap();
        let res = self
            .client
            .get(complete_url)
            .body(serialized_request)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .send()
            .await
            .map_err(errors::Error::RequestError)?
            .text()
            .await
            .map_err(errors::Error::RequestError)?;

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn make_request() {
        let bill = Billine::new("".into(), "".into());

        bill.signed_get_request(
            "/payment/form",
            models::RequestIframe {
                merchant: "".into(),
                order: "TEST".into(),
                amount: Default::default(),
                currency: "USD".into(),
                item_name: "TEST".into(),
                first_name: "TEST".into(),
                last_name: "TEST".into(),
                user_id: "TEST".into(),
                payment_url: "TEST".into(),
                country: "TEST".into(),
                ip: "212.10.20.75".into(),
                custom: "TEST".into(),
                email: "TEST".into(),
                phone: "TEST".into(),
                address: "TEST".into(),
                city: "TEST".into(),
                post_code: "TEST".into(),
                region: "TEST".into(),
                lang: models::Language::En,
                cpf: None,
            },
        )
        .await
        .unwrap();
    }
}
