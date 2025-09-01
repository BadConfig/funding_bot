pub mod callbacks;
pub mod commands;

use std::sync::{Arc, Mutex};

use teloxide::{dispatching::UpdateHandler, macros::BotCommands, prelude::*};

use crate::PositionCandidate;

#[derive(Debug, Clone)]
pub struct HandlerContext {
    pub position_candidates: Arc<Mutex<Vec<PositionCandidate>>>,
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "return top arbitrages by apy")]
    TopApy { number: u8 },
    //#[command(description = "handle a username and an age.", parse_with = "split")]
    //UsernameAndAge { username: String, age: u8 },
}

pub fn schema() -> UpdateHandler<anyhow::Error> {
    // Dispatcher tree
    Update::filter_message()
        .branch(teloxide::filter_command::<Command, _>().endpoint(commands::handler))
        .branch(Update::filter_callback_query().endpoint(callbacks::handler))
}
