pub mod admin;
pub mod general;
pub mod schedule;

use std::sync::Arc;

use anyhow::Result;
use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDate, Weekday};
use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
    utils::command::BotCommands,
};

use crate::{
    button_prefix::ButtonPrefix,
    config::AppConfig,
    utils::{database::Database, sql::models::get_user_entry_by_id, time::TIME_OFFSET_SECONDS},
};

enum KeyboardWeek {
    Next,
    Current,
}

/// Creates a keyboard made by buttons in a big column.
fn make_keyboard(kbd_week: KeyboardWeek) -> Result<InlineKeyboardMarkup> {
    let dt = crate::utils::time::now()?;

    let week = match kbd_week {
        KeyboardWeek::Next => dt.iso_week().week() + 1,
        KeyboardWeek::Current => dt.iso_week().week(),
    };

    let monday = NaiveDate::from_isoywd_opt(dt.year(), week, Weekday::Mon)
        .unwrap()
        .and_hms_opt(8, 0, 0)
        .unwrap()
        .and_local_timezone(FixedOffset::east_opt(TIME_OFFSET_SECONDS).unwrap())
        .unwrap();

    let weekdays = (0..=6)
        .map(|i| monday + Duration::hours(24 * i))
        .filter(|dt| dt.weekday() != Weekday::Sun)
        .collect::<Vec<_>>();

    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];

    for days in weekdays.chunks(3) {
        let row = days
            .iter()
            .map(|day| {
                let prefix = format!("{}", ButtonPrefix::TimetableWeekday);
                let text = day
                    .format_localized("%A", chrono::Locale::ru_RU)
                    .to_string();

                let data = format!("{}:{}", prefix, day.to_rfc3339());

                InlineKeyboardButton::callback(text, data)
            })
            .collect();

        keyboard.push(row);
    }

    Ok(InlineKeyboardMarkup::new(keyboard))
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "<b>Команды для студентов</b>"
)]
pub enum TimetableCommand {
    #[command(description = "Просмотр расписания за вчера.")]
    Yesterday,
    #[command(description = "Просмотр расписания на сегодня.")]
    Today,
    #[command(description = "Просмотр расписания на завтра.")]
    Tomorrow,
    #[command(description = "Выбрать день со следующий недели.")]
    NextWeek,
    #[command(description = "Выбрать день на неделе.")]
    ThisWeek,
}

pub async fn timetable_commands_handler(
    _config: Arc<AppConfig>,
    db: Database,
    bot: Bot,
    msg: Message,
    cmd: TimetableCommand,
) -> Result<()> {
    let dt = crate::utils::time::now()?;

    // user must exist since we do filtering with dptree
    let author = msg.from().unwrap();
    let author_id = i64::try_from(author.id.0)?;

    let user_entry = get_user_entry_by_id(db.pool.as_ref(), author_id).await?;

    match cmd {
        TimetableCommand::Yesterday => {
            let dt = dt - Duration::hours(24);
            self::schedule::command_handler(&db, &bot, dt, &user_entry.major_id, &msg.chat).await?;
        }

        TimetableCommand::Today => {
            self::schedule::command_handler(&db, &bot, dt, &user_entry.major_id, &msg.chat).await?;
        }

        TimetableCommand::Tomorrow => {
            let dt = dt + Duration::hours(24);
            self::schedule::command_handler(&db, &bot, dt, &user_entry.major_id, &msg.chat).await?;
        }

        TimetableCommand::ThisWeek => {
            let kb = make_keyboard(KeyboardWeek::Current)?;

            bot.send_message(msg.chat.id, "Выберите интересующий вас день текущей недели")
                .reply_markup(kb)
                .await?;
        }

        TimetableCommand::NextWeek => {
            let kb = make_keyboard(KeyboardWeek::Next)?;

            bot.send_message(
                msg.chat.id,
                "Выберите интересующий вас день следующей недели",
            )
            .reply_markup(kb)
            .await?;
        }
    };

    Ok(())
}

pub async fn timetable_callback_handler(db: Database, bot: Bot, q: CallbackQuery) -> Result<()> {
    let Some(data) = q.data else {
        return Ok(())
    };

    let Some((button_prefix, date_rfc3339)) = data.split_once(':') else {
        return Ok(())
    };

    if button_prefix != format!("{}", ButtonPrefix::TimetableWeekday) {
        return Ok(());
    };

    bot.answer_callback_query(q.id).await?;

    let dt: DateTime<FixedOffset> = DateTime::parse_from_rfc3339(date_rfc3339)?;

    let author = q.from;
    let author_id = i64::try_from(author.id.0)?;

    let user_entry = get_user_entry_by_id(db.pool.as_ref(), author_id).await?;

    if let Some(Message { id, chat, .. }) = q.message {
        self::schedule::button_handler_known_chat(&db, &bot, dt, &user_entry.major_id, &chat, id)
            .await?;
    } else if let Some(id) = q.inline_message_id {
        self::schedule::button_handler_unknown_chat(&db, &bot, dt, &user_entry.major_id, id)
            .await?;
    }

    Ok(())
}
