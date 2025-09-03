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
                "currency:          {}\n\
total funding:  {}%h\n\
APY:                {}%\n\
Spread:         {}%\n\
Long on:        {:?}\n\
Long Funding:   {}\n\
Long OI:        {}$\n\
Short on:       {:?}\n\
Short Funding:  {}\n\
Short OI:       {}$\n",
                v.currency_name,
                (v.total_funding * Decimal::from(100)).round_dp(6),
                v.apy.round_dp(2),
                v.spread.unwrap_or_default().round_dp(6),
                v.long_on,
                (v.long_funding * Decimal::from(100)).round_dp(6),
                format_short(v.oi_long.unwrap_or_default().round_dp(0)),
                v.short_on,
                (v.short_funding * Decimal::from(100)).round_dp(6),
                format_short(v.oi_short.unwrap_or_default().round_dp(0)),
            )
        })
        .join("---------------------------------\n")
}

fn format_short(n: Decimal) -> String {
    let thousand = Decimal::from(1000);
    let million = Decimal::from(1_000_000);
    let billion = Decimal::from(1_000_000_000);

    if n.abs() >= billion {
        format!("{:.2}B", n / billion)
    } else if n.abs() >= million {
        format!("{:.2}M", n / million)
    } else if n.abs() >= thousand {
        format!("{:.2}K", n / thousand)
    } else {
        format!("{}", n.round_dp(2))
    }
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
                            long_funding: long_dex.funding_rate,
                            short_funding: short_dex.funding_rate,
                            currency_name: currency_name.clone(),
                            long_on: long_dex.exchange.clone(),
                            long_market: long_dex.market_name.clone(),
                            short_on: short_dex.exchange.clone(),
                            short_market: short_dex.market_name.clone(),
                            total_funding: total_funding.abs(),
                            apy: total_funding.abs() * Decimal::from(24 * 365 * 100),
                            oi_long: long_dex.open_interest,
                            oi_short: short_dex.open_interest,
                            spread: long_dex
                                .best_bid
                                .zip(short_dex.best_ask)
                                .map(|(l, s)| (l - s) * Decimal::from(100) / l),
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
