//curl -X GET https://api.prod.paradex.trade/v1/markets/summary?market=BTC-USD-PERP \
// -H 'Accept: application/json'

use anyhow::Context;
use itertools::Itertools;
use serde_json::{Value, json};

use crate::{Exchange, Funding};

pub async fn request_fundings() -> anyhow::Result<Vec<Funding>> {
    reqwest::Client::new()
        .post("https://api.hyperliquid.xyz/info")
        .json(&json!({
            "type": "metaAndAssetCtxs",
        }))
        .send()
        .await?
        .json()
        .await
        .map(|m: Value| {
            let meta = m
                .get(0)
                .unwrap()
                .get("universe")
                .unwrap()
                .as_array()
                .unwrap();
            let stats = m.get(1).unwrap().as_array().unwrap();

            meta.iter()
                .zip(stats.iter())
                .map(|(meta, stats)| Funding {
                    best_ask: None,
                    best_bid: None,
                    exchange: Exchange::Hyperliquid,

                    market_name: meta.get("name").unwrap().as_str().unwrap().to_string(),
                    currency_name: meta.get("name").unwrap().as_str().unwrap().to_string(),

                    funding_rate: stats
                        .get("funding")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .parse()
                        .unwrap(),
                    open_interest: Some(
                        stats
                            .get("openInterest")
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .parse()
                            .unwrap(),
                    ),
                })
                .collect_vec()
        })
        .context("paradex failed to fetch data")
}
