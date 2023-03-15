use std::sync::Arc;

use anyhow::Result;
use teloxide::{prelude::*, types::Update};

use crate::button_prefix::ButtonPrefix;
use crate::config::AppConfig;
use crate::utils::database::Database;

mod button_prefix;
mod config;
mod handlers;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    simple_logger::init_with_env().unwrap();

    log::info!("Starting...");

    let config = Arc::new(AppConfig::figment());

    let pool = Database::create_pool(&config.database.url).await?;
    let db = Database::new(Arc::new(pool));

    let bot = Bot::new(&config.telegram.token);

    let commands_handler = Update::filter_message()
        .branch(
            dptree::entry()
                .filter_command::<handlers::general::GeneralCommand>()
                .endpoint(handlers::general::general_commands_handler),
        )
        .branch(
            dptree::entry()
                .filter_async(|bot: Bot, msg: Message, db: Database| async move {
                    handlers::schedule::filter_predicate(&bot, &msg, &db)
                        .await
                        .unwrap()
                })
                .filter_command::<handlers::TimetableCommand>()
                .endpoint(handlers::timetable_commands_handler),
        )
        .branch(
            dptree::filter(|cfg: Arc<AppConfig>, msg: Message| {
                if let Some(sender) = msg.from() {
                    cfg.telegram.owner_ids.contains(&sender.id.0)
                } else {
                    false
                }
            })
            .filter_command::<handlers::admin::AdminCommand>()
            .endpoint(handlers::admin::commands_handler),
        );

    let callback_handler = Update::filter_callback_query()
        .branch(
            dptree::filter(|q: CallbackQuery| match q.data {
                Some(data) => data.starts_with(&format!("{}", ButtonPrefix::SetMajor)),
                None => false,
            })
            .endpoint(handlers::general::set_major_callback_handler),
        )
        .branch(
            dptree::filter(|q: CallbackQuery| match q.data {
                Some(data) => data.starts_with(&format!("{}", ButtonPrefix::TimetableWeekday)),
                None => false,
            })
            .endpoint(handlers::timetable_callback_handler),
        );

    let handler = dptree::entry()
        .branch(commands_handler)
        .branch(callback_handler);

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![config, db])
        .default_handler(|upd| async move {
            log::warn!("Unhandled update: #{}", upd.id);
        })
        .error_handler(LoggingErrorHandler::with_custom_text(
            "An error has occurred in the dispatcher",
        ))
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
