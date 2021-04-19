//! # RxNorm
//! Wrapper for the RxNav [RxNorm API](https://rxnav.nlm.nih.gov/RxNormAPIs.html)

use reqwest::{Client, Error, Response};
use tokio::time::{sleep, Duration};

const RXNAV_URL: &str = "https://rxnav.nlm.nih.gov/REST/rxcui.json";

pub struct RxNormClient {
    client: reqwest::Client,
    normalize: bool,
}

impl RxNormClient {
    pub fn new(client: reqwest::Client, normalize: bool) -> Self {
        Self { client, normalize }
    }

    /// Finds the RxCUI for a givin string.
    ///
    /// # Examples
    /// ```rust
    ///use reqwest::Client;
    /// use rxnormalizer::RxNormClient;
    ///
    /// #[tokio::main]
    ///async fn main() {
    ///
    /// let http_client = Client::new();
    /// let rx_client = RxNormClient::new(http_client, true);
    ///let vit_c: &String = &String::from("vit-c");
    ///
    /// // Calling RxNav
    ///let actual: Vec<i32> = rx_client.find_rxcui(&vit_c).await.unwrap().expect("Could not find vit-c");
    ///let expected: Vec<i32> = vec![1088438, 1151];
    ///assert_eq!(expected, actual)
    /// }
    /// ```
    ///
    ///

    pub async fn find_rxcui(&self, drug: &String) -> Result<Option<Vec<i32>>, &'static str> {
        let mode = if self.normalize { "2" } else { "0" };
        let result = make_call(&drug, &self.client, &String::from(mode)).await;
        let res = match result {
            Ok(res) => res,
            Err(e) => {
                println!(
                    "Caught an error of kind {}, going to wait 2 seconds and try again",
                    e.to_string()
                );
                sleep(Duration::from_secs(2)).await;
                make_call(&drug, &self.client, &String::from(mode))
                    .await
                    .unwrap()
            }
        };
        let status = res.status();
        let body = res.text().await.unwrap();
        if status.is_success() {
            let rxnorm = json::parse(&body).unwrap();
            let result: String = rxnorm["idGroup"]["rxnormId"]
                .dump()
                .replace(&['[', ']', '\"'][..], "");
            if !result.eq("null") {
                let ids: Vec<i32> = result
                    .split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .map(|s| s.parse().unwrap())
                    .collect();
                return Ok(Some(ids));
            }
            return Ok(None);
        } else {
            Err("RxNav returned an error")
        }
    }
}

async fn make_call(drug: &String, client: &Client, mode: &String) -> Result<Response, Error> {
    let result = client
        .get(RXNAV_URL)
        .query(&[("name", &drug), ("search", &mode)])
        .send()
        .await;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_vit_c_with_normalizer() {
        let http_client = reqwest::Client::new();
        let rx_client = RxNormClient::new(http_client, true);
        let vit_c: &String = &String::from("vit-c");
        let expected: Vec<i32> = vec![1088438, 1151];
        let actual: Vec<i32> = rx_client
            .find_rxcui(&vit_c)
            .await
            .unwrap()
            .expect("Something went wrong");
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_vit_c_without_normalizer() {
        let http_client = reqwest::Client::new();
        let rx_client = RxNormClient::new(http_client, false);
        let vit_c: &String = &String::from("vit-c");
        let actual: Option<Vec<i32>> = rx_client.find_rxcui(&vit_c).await.unwrap();
        assert!(actual.is_none());
    }
}
