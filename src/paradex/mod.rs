//curl -X GET https://api.prod.paradex.trade/v1/markets/summary?market=BTC-USD-PERP \
// -H 'Accept: application/json'

use anyhow::Context;
use itertools::Itertools;
use rust_decimal::Decimal;
use serde_json::Value;

use crate::{Exchange, Funding};

pub async fn request_fundings() -> anyhow::Result<Vec<Funding>> {
    reqwest::get("https://api.prod.paradex.trade/v1/markets/summary?market=ALL")
        .await?
        .json()
        .await
        .map(|m: Value| {
            m.get("results")
                .unwrap()
                .as_array()
                .unwrap()
                .iter()
                .filter(|v| {
                    v.get("symbol")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .split("-USD-PERP")
                        .next()
                        .is_some()
                })
                .filter(|v| v.get("symbol").unwrap().as_str().unwrap() != "USDC")
                .map(|v| Funding {
                    best_ask: None,
                    best_bid: None,
                    exchange: Exchange::Paradex,
                    open_interest: Some(
                        v.get("open_interest")
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .parse()
                            .unwrap(),
                    ),

                    market_name: v.get("symbol").unwrap().as_str().unwrap().to_string(),
                    currency_name: v
                        .get("symbol")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .split("-USD-PERP")
                        .next()
                        .unwrap()
                        .to_string(),
                    funding_rate: v
                        .get("funding_rate")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .parse::<Decimal>()
                        .unwrap()
                        / Decimal::from(8),
                })
                .collect_vec()
        })
        .context("paradex failed to fetch data")
}
