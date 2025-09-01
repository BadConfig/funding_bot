use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{PositionCandidate, extended, hyperliquid, paradex};
use itertools::Itertools;
use rust_decimal::Decimal;

pub fn candidates_to_string(c: &[PositionCandidate]) -> String {
    c.iter()
        .map(|v| {
            format!(
                "currency:          {}\ntotal funding:      {}\nAPY:            {}%\nLong on:           {:?}\nMarket:       {}\nOpen Interest:      {:?}\nShort on:             {:?}\nMarket:       {}\nOpen Interest:          {:?}\n",
                v.currency_name,
                v.total_funding.round_dp(6),
                v.apy.round_dp(2),
                v.long_on,
                v.long_market,
                v.oi_long.unwrap_or_default().round_dp(0),
                v.short_on,
                v.short_market,
                v.oi_short.unwrap_or_default().round_dp(0),
            )
        })
        .join("---------------------------------\n")
}

pub async fn fill_fundings(shared: Arc<Mutex<Vec<PositionCandidate>>>) -> anyhow::Result<()> {
    loop {
        log::info!("Tick");
        let extended_funding = extended::request_fundings()
            .await?
            .into_iter()
            .sorted_by_key(|v| v.currency_name.clone());
        let paradex_funding = paradex::request_fundings()
            .await?
            .into_iter()
            .sorted_by_key(|v| v.currency_name.clone());

        let hyperliquid_funding = hyperliquid::request_fundings()
            .await?
            .into_iter()
            .sorted_by_key(|v| v.currency_name.clone());

        log::info!("extended len {}", extended_funding.len());
        log::info!("paradex len {}", paradex_funding.len());
        log::info!("hyperliquid len {}", hyperliquid_funding.len());

        let merged = vec![extended_funding, paradex_funding, hyperliquid_funding]
            .into_iter()
            .kmerge_by(|a, b| a.currency_name < b.currency_name)
            .chunk_by(|v| v.currency_name.clone());

        let results: Vec<PositionCandidate> = merged
            .into_iter()
            .flat_map(|(currency_name, fundings)| {
                fundings
                    .into_iter()
                    .combinations(2) // all pairs
                    .map(move |f| {
                        let v1 = &f[0];
                        let v2 = &f[1];
                        let total_funding = v1.funding_rate - v2.funding_rate;
                        let (long_dex, short_dex) = if total_funding.is_sign_positive() {
                            (v1, v2)
                        } else {
                            (v2, v1)
                        };
                        PositionCandidate {
                            currency_name: currency_name.clone(),
                            long_on: long_dex.exchange.clone(),
                            long_market: long_dex.market_name.clone(),
                            short_on: short_dex.exchange.clone(),
                            short_market: short_dex.market_name.clone(),
                            total_funding: total_funding.abs(),
                            apy: total_funding.abs() * Decimal::from(24 * 365 * 100),
                            oi_long: long_dex.open_interest,
                            oi_short: short_dex.open_interest,
                        }
                    })
            })
            .sorted_by_key(|p| -p.apy)
            .collect();

        {
            let mut f = shared.lock().unwrap();
            *f = results;
        }
        log::info!("Going to sleep");
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
