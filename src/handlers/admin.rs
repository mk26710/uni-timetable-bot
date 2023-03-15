use std::sync::Arc;

use anyhow::Result;
use rand::Rng;
use teloxide::{prelude::*, utils::command::BotCommands};

use crate::{config::AppConfig, utils::database::Database};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Admin commands")]
pub enum AdminCommand {
    #[command(description = "Help for admin only commands")]
    AdminHelp,
    #[command(parse_with = "split", description = "generate a number within range")]
    Rand { from: u64, to: u64 }
}

pub async fn commands_handler(
    _cfg: Arc<AppConfig>,
    _db: Database,
    bot: Bot,
    msg: Message,
    cmd: AdminCommand,
) -> Result<()> {
    match cmd {
        AdminCommand::AdminHelp => {
            bot.send_message(msg.chat.id, AdminCommand::descriptions().to_string())
                .await?;
        }

        AdminCommand::Rand { from, to } => {
            let mut rng = rand::rngs::OsRng::default();
            let num = rng.gen_range(from..=to);

            bot.send_message(msg.chat.id, format!("{num}")).await?;
        }
    }

    Ok(())
}
