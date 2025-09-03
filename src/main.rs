use std::sync::{Arc, Mutex};

use bot::HandlerContext;
use log::LevelFilter;
use rust_decimal::Decimal;
use teloxide::prelude::*;

pub mod bot;
pub mod extended;
pub mod hyperliquid;
pub mod paradex;
pub mod process_fundings;

#[derive(Debug, Clone)]
pub enum Exchange {
    Paradex,
    Extended,
    Hyperliquid,
    Vest,
}

#[derive(Debug, Clone)]
pub struct Funding {
    currency_name: String,
    funding_rate: Decimal,
    market_name: String,
    exchange: Exchange,
    open_interest: Option<Decimal>,
    best_bid: Option<Decimal>,
    best_ask: Option<Decimal>,
}

#[derive(Debug, Clone)]
pub struct PositionCandidate {
    currency_name: String,
    total_funding: Decimal,
    apy: Decimal,
    long_on: Exchange,
    long_funding: Decimal,
    long_market: String,
    short_on: Exchange,
    short_funding: Decimal,
    short_market: String,
    oi_long: Option<Decimal>,
    oi_short: Option<Decimal>,
    spread: Option<Decimal>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::formatted_builder()
        .default_format()
        .filter_level(LevelFilter::Debug)
        .init();
    log::info!("Starting throw dice bot...");

    let bot = Bot::from_env();

    let fundings = Arc::new(Mutex::new(Vec::<PositionCandidate>::new()));

    let h = {
        let fundings = fundings.clone();
        tokio::spawn(async move {
            process_fundings::fill_fundings(fundings.clone())
                .await
                .unwrap()
        })
    };

    let ctx = Arc::new(HandlerContext {
        position_candidates: fundings.clone(),
    });

    {
        tokio::spawn(async move {
            Dispatcher::builder(bot, bot::schema())
                .dependencies(dptree::deps![ctx]) // 👈 inject app context here
                .enable_ctrlc_handler()
                .build()
                .dispatch()
                .await;
        });
    }

    h.await.unwrap();
    Ok(())
}
