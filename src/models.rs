use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use md5::{Digest, Md5};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use utoipa::ToSchema;

fn sort_alphabetically<T: Serialize, S: serde::Serializer>(
    value: &T,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let value = serde_json::to_value(value).map_err(serde::ser::Error::custom)?;
    value.serialize(serializer)
}

#[derive(Serialize)]
struct SortAlphabetically<T: Serialize>(#[serde(serialize_with = "sort_alphabetically")] T);

pub fn serialize<T: Serialize>(data: &T) -> String {
    Itertools::intersperse(
        serde_json::to_value(&SortAlphabetically(data))
            .unwrap()
            .as_object()
            .unwrap()
            .values()
            .filter_map(|v| match v {
                serde_json::Value::Null => None,
                serde_json::Value::Bool(b) => Some(b.to_string()),
                serde_json::Value::Number(n) => Some(n.to_string()),
                serde_json::Value::String(s) => Some(s.to_string()),
                serde_json::Value::Array(a) => {
                    Some(serde_json::Value::Array(a.to_vec()).to_string())
                }
                serde_json::Value::Object(o) => {
                    Some(serde_json::Value::Object(o.clone()).to_string())
                }
            }),
        ":".to_string(),
    )
    .collect::<String>()
}

pub fn md5_signature<T: Serialize>(data: &T, secret_key: &str) -> String {
    let params = serialize(data);
    let input = format!("{}:{}", params, secret_key);

    let mut hasher = Md5::new();

    hasher.update(input.as_bytes());

    let result = hasher.finalize();

    let signature = STANDARD.encode(result);

    signature
}

pub fn sha256_signature<T: Serialize>(data: &T, secret_key: &str) -> String {
    let params = serialize(data);
    let input = format!("{}:{}", params, secret_key);

    let mut hasher = Sha256::new();

    hasher.update(input.as_bytes());

    let result = hasher.finalize();

    let signature = STANDARD.encode(result);

    signature
}

#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Success,
    Fail,
}

/// Callback model for Iframe/h2h
#[derive(Deserialize, Serialize, ToSchema)]
pub struct CallbackIframe {
    pub co_inv_id: String,
    #[serde(with = "date")]
    pub co_inv_crt: DateTime<Utc>,
    #[serde(with = "date")]
    pub co_inv_prc: DateTime<Utc>,
    pub co_inv_st: Status,
    pub co_order_no: String,
    pub co_amount: Option<Decimal>,
    pub co_to_wlt: Option<Decimal>,
    pub co_cur: Option<String>,
    pub co_merchant_id: String,
    pub co_merchant_uuid: String,
    pub co_sign: String,
    pub co_base_amount: Option<Decimal>,
    pub co_base_currency: Option<String>,
    pub co_rate: Option<Decimal>,
}

#[derive(Deserialize, Serialize, ToSchema, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    En,
    Ua,
}

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub struct RequestIframe {
    pub merchant: String,
    pub order: String,
    pub amount: Decimal,
    pub currency: String,
    pub item_name: String,
    pub first_name: String,
    pub last_name: String,
    pub user_id: String,
    pub payment_url: String,
    pub country: String,
    pub ip: String,
    pub custom: String,
    pub email: String,
    pub phone: String,
    pub address: String,
    pub city: String,
    pub post_code: String,
    pub region: String,
    pub lang: Language,
    pub cpf: Option<String>,
}

mod date {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt = NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)?;
        Ok(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    #[derive(Serialize, Deserialize, Default, Debug)]
    struct TestStruct {
        merchant: String,
        method: u64,
        payout_id: String,
        account: String,
        amount: f64,
        currency: String,
    }

    #[test]
    fn test_serialize() {
        println!(
            "{:?}",
            serde_json::to_value(&SortAlphabetically(&TestStruct {
                merchant: "merch".into(),
                method: 5,
                payout_id: "ID".into(),
                account: "acc".into(),
                amount: 35.45,
                currency: "USD".into()
            }))
            .unwrap()
            .as_object()
            .unwrap()
            .values()
            .filter_map(|v| match v {
                serde_json::Value::Null => None,
                serde_json::Value::Bool(b) => Some(b.to_string()),
                serde_json::Value::Number(n) => Some(n.to_string()),
                serde_json::Value::String(s) => Some(s.to_string()),
                serde_json::Value::Array(a) =>
                    Some(serde_json::Value::Array(a.to_vec()).to_string()),
                serde_json::Value::Object(o) =>
                    Some(serde_json::Value::Object(o.clone()).to_string()),
            })
            .intersperse(":".to_string())
            .collect::<String>()
        );
    }

    #[test]
    fn test_md5_signature() {
        let data = TestStruct {
            merchant: "M1VJDHSI6DYXS".into(),
            method: 1,
            payout_id: "000002".into(),
            account: "5300111122223333".into(),
            amount: 1.19,
            currency: "UAH".into(),
        };
        let signature = md5_signature(&data, "SecRetKey0123");

        assert_eq!(signature, "HyTFPDEwJjcnCMmD/AE5wg==");
    }
}
