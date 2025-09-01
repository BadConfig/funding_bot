use std::sync::Arc;

use anyhow::Result;
use itertools::Itertools;
use teloxide::{dispatching::dialogue::GetChatId, prelude::*, utils::command::BotCommands};

use super::{Command, HandlerContext};

pub async fn handler(bot: Bot, cb: CallbackQuery, context: Arc<HandlerContext>) -> Result<()> {
    //let fundings = context.position_candidates.clone();

    todo!();
    //  match cmd {
    //      Command::Help => {
    //          bot.send_message(cb.chat_id().unwrap(), Command::descriptions().to_string())
    //              .await?;
    //      }
    //      Command::TopApy { number } => {
    //          let chat_id = ChatId(-1002923225852);
    //          if cb.chat_id().as_ref() != Some(&chat_id) {
    //              return Ok(());
    //          }

    //          let f = {
    //              let fundings = &fundings.lock().unwrap();
    //              candidates_to_string(
    //                  fundings
    //                      .iter()
    //                      .take(number.into())
    //                      .cloned()
    //                      .collect_vec()
    //                      .as_slice(),
    //              )
    //          };
    //          bot.send_message(chat_id, f).await?;
    //      }
    //  }

    Ok(())
}
