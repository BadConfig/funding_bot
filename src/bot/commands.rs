use std::sync::Arc;

use anyhow::Result;
use itertools::Itertools;
use teloxide::{dispatching::dialogue::GetChatId, prelude::*, utils::command::BotCommands};

use crate::process_fundings::candidates_to_string;

use super::{Command, HandlerContext};

pub async fn handler(
    bot: Bot,
    msg: Message,
    cmd: Command,
    context: Arc<HandlerContext>,
) -> Result<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat_id().unwrap(), Command::descriptions().to_string())
                .await?;
        }
        Command::TopApy { number } => {
            let chat_id = ChatId(-1002923225852);
            if msg.chat_id().as_ref() != Some(&chat_id) {
                return Ok(());
            }

            let f = {
                let fundings = &context.position_candidates.lock().unwrap();
                candidates_to_string(
                    fundings
                        .iter()
                        .take(number.into())
                        .cloned()
                        .collect_vec()
                        .as_slice(),
                )
            };
            bot.send_message(chat_id, f).await?;
        }
    }

    Ok(())
}
