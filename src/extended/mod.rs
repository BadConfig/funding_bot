// /api/v1/info/markets

use anyhow::Context;
use itertools::Itertools;
use reqwest::header::{ACCEPT, USER_AGENT};
use serde_json::Value;

use crate::{Exchange, Funding};

pub async fn request_fundings() -> anyhow::Result<Vec<Funding>> {
    reqwest::Client::new()
        .get("https://api.starknet.extended.exchange/api/v1/info/markets")
        .header(USER_AGENT, "curl/8.7.1")
        .header(ACCEPT, "*/*")
        .send()
        .await?
        .json()
        .await
        .map(|m: Value| {
            m.get("data")
                .unwrap()
                .as_array()
                .unwrap()
                .iter()
                .filter(|v| {
                    v.get("name")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .split("-USD")
                        .next()
                        .is_some()
                })
                .map(|v| Funding {
                    best_ask: None,
                    best_bid: None,
                    exchange: Exchange::Extended,
                    open_interest: Some(
                        v.get("marketStats")
                            .unwrap()
                            .get("openInterest")
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .parse()
                            .unwrap(),
                    ),

                    market_name: v.get("name").unwrap().as_str().unwrap().to_string(),
                    currency_name: v.get("assetName").unwrap().as_str().unwrap().to_string(),
                    funding_rate: v
                        .get("marketStats")
                        .unwrap()
                        .get("fundingRate")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .parse()
                        .unwrap(),
                })
                .collect_vec()
        })
        .context("extended failed to fetch data")
}
