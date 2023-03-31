use std::time::{SystemTime, UNIX_EPOCH};
use querystring;
use ring::hmac;
use data_encoding::HEXLOWER;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub order_id: u128,
    pub symbol: String,
    pub status: String,
    #[serde(deserialize_with  = "fix_numbers")]
    pub price: f64,
    #[serde(deserialize_with  = "fix_numbers")]
    pub orig_qty: f64,
    pub orig_type: String,
    pub side: String
}

pub struct BinanceAPI {
    key: String,
    secret_key: String,
    client: reqwest::Client,
    domain: String
}

impl BinanceAPI {
    fn get_signature(&self, query_params: &Vec<(&str, &str)>) -> String {
        let mut query_params_for_sig_str = querystring::stringify(query_params.clone());
        query_params_for_sig_str = query_params_for_sig_str.trim_end_matches('&').to_string();
    
        let signed_key = hmac::Key::new(hmac::HMAC_SHA256, self.secret_key.as_bytes());
        let signature = hmac::sign(&signed_key, query_params_for_sig_str.as_bytes());
        let signature_str = HEXLOWER.encode(signature.as_ref());
        
        signature_str
    }
}

impl BinanceAPI {
    pub async fn get_open_orders(&self) -> Vec<Order> {
        let domain = &self.domain;
        let url = "/fapi/v1/openOrders";
        
        let ts = self.get_server_time().await;
        let ts_str = ts.to_string();
    
        let mut query_params = vec![
            ("timestamp", ts_str.as_str()),
            ("recvWindow", "10000")
        ];
        
        let signature = self.get_signature(&query_params);
        query_params.push(("signature", signature.as_str()));
    
        let rj = self.client.get(format!("{domain}{url}"))
            .header("X-MBX-APIKEY", self.key.as_str())
            .query::<Vec<(&str, &str)>>(&query_params)
            .send()
            .await
            .unwrap()
            .json::<Vec<Order>>()
            .await
            .unwrap();

        rj
    }
}

impl BinanceAPI {
    pub async fn close_all_orders(&self, symbol: &str) -> bool {
        let domain = &self.domain;
        let url = "/fapi/v1/allOpenOrders ";
        
        let ts = self.get_server_time().await;
        let ts_str = ts.to_string();

        let mut query_params = vec![
            ("timestamp", ts_str.as_str()),
            ("recvWindow", "10000"),
            ("symbol", symbol),
        ];

        let signature = self.get_signature(&query_params);
        query_params.push(("signature", signature.as_str()));

        let response = self.client.delete(format!("{domain}{url}"))
            .header("X-MBX-APIKEY", self.key.as_str())
            .query::<Vec<(&str, &str)>>(&query_params)
            .send()
            .await
            .unwrap();
        let status_code = response.status().to_string();
        status_code == "200 OK"
    }
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ServerTimeResponse {
    pub server_time: u128
}

impl BinanceAPI {
    pub async fn get_server_time(&self) -> u128 {
        let domain = &self.domain;
        let url = "/fapi/v1/time";

        let response = self.client.get(format!("{domain}{url}"))
            .header("X-MBX-APIKEY", self.key.as_str())
            .send()
            .await
            .unwrap()
            .json::<ServerTimeResponse>()
            .await
            .unwrap()
        ;

        return response.server_time;
    }
}

pub fn new(key: String, secret_key: String) -> BinanceAPI {
    BinanceAPI { 
        key, 
        secret_key,
        client: reqwest::Client::new(),
        domain: String::from("https://fapi.binance.com")
    }
}

fn _get_os_timestamp() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

        since_the_epoch.as_millis()
}


pub fn fix_numbers<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    String::deserialize(deserializer)
        .and_then(|string| {
            let mut trimmed_str = string.trim_end_matches('0').trim_end_matches('.').to_string();
            if trimmed_str == "" {
                trimmed_str = String::from("0");
            }
            trimmed_str.parse::<f64>().map_err(|err| Error::custom(err.to_string()))
        }
    )
}