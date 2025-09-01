use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use log::LevelFilter;
use rust_decimal::Decimal;
use teloxide::{dispatching::dialogue::GetChatId, prelude::*, types::Recipient};

pub mod extended;
pub mod hyperliquid;
pub mod paradex;

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
    long_market: String,
    short_on: Exchange,
    short_market: String,
}

fn candidates_to_string(c: &[PositionCandidate]) -> String {
    c.into_iter()
        .map(|v| {
            format!(
                "currency: {}\ntotal funding: {}\nAPY: {}%\nlong: {:?}({})\nshort: {:?}({})\n",
                v.currency_name,
                v.total_funding.round_dp(6),
                v.apy.round_dp(2),
                v.long_on,
                v.long_market,
                v.short_on,
                v.short_market
            )
        })
        .join("-----------------------\n")
}

use itertools::Itertools;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::formatted_builder()
        .default_format()
        .filter_level(LevelFilter::Debug)
        .init();
    log::info!("Starting throw dice bot...");

    let bot = Bot::from_env();

    let fundings = Arc::new(Mutex::new(Vec::<PositionCandidate>::new()));

    {
        let fundings = fundings.clone();
        tokio::spawn(async move { fill_fundings(fundings.clone()).await.unwrap() });
    }

    //bot.send_message(chat_id, "test").await?;
    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        let fundings = fundings.clone();
        async move {
            let chat_id = ChatId(-1002923225852);
            if msg.chat_id().as_ref() != Some(&chat_id) {
                return Ok(());
            }

            if let Some(_text) = msg.text() {
                let f = {
                    let fundings = &fundings.lock().unwrap();
                    candidates_to_string(
                        fundings
                            .iter()
                            .take(5)
                            .map(|v| v.clone())
                            .collect_vec()
                            .as_slice(),
                    )
                };
                bot.send_message(chat_id, f).await?;
            }
            Ok(())
        }
    })
    .await;
    Ok(())
}

async fn fill_fundings(shared: Arc<Mutex<Vec<PositionCandidate>>>) -> anyhow::Result<()> {
    loop {
        log::info!("Tick");
        let extended_funding = extended::request_fundings().await?;
        let paradex_funding = paradex::request_fundings().await?;

        log::info!("extended len {}", extended_funding.len());
        log::info!("paradex len {}", paradex_funding.len());

        let mut fundings = Vec::new();

        let hm = extended_funding
            .into_iter()
            .map(|v| (v.currency_name.clone(), v))
            .collect::<HashMap<_, _>>();

        for y in paradex_funding {
            match hm.get(&y.currency_name) {
                Some(x) => {
                    let total_funding = (x.funding_rate - y.funding_rate).abs();
                    let long_on_x = x.funding_rate > y.funding_rate;
                    fundings.push(PositionCandidate {
                        currency_name: x.currency_name.clone(),
                        total_funding,
                        apy: total_funding * Decimal::from(876000),
                        long_on: if long_on_x {
                            Exchange::Extended
                        } else {
                            Exchange::Paradex
                        },
                        short_on: if long_on_x {
                            Exchange::Paradex
                        } else {
                            Exchange::Extended
                        },
                        long_market: if long_on_x {
                            x.market_name.clone()
                        } else {
                            y.market_name.clone()
                        },
                        short_market: if long_on_x {
                            y.market_name
                        } else {
                            x.market_name.clone()
                        },
                    });
                }
                None => continue,
            }
        }

        {
            let mut f = shared.lock().unwrap();
            *f = fundings;
        }
        log::info!("Going to sleep");
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}

//    let merged: Vec<_> = vec![a, b, c]
//        .into_iter()
//        .kmerge()   // merge k sorted iterators
//        .dedup()    // remove duplicates
//        .collect();
