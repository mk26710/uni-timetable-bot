use anyhow::Result;
use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode},
    utils::command::BotCommands,
};

use crate::{utils::{
    database::Database,
    sql::types::{MajorEntry, UserEntry},
}, button_prefix::ButtonPrefix};

use super::TimetableCommand;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "<b>Общедоступные команды</b>"
)]
pub enum GeneralCommand {
    #[command(description = "Отображает список команд")]
    Help,
    #[command(description = "Установить свою группу")]
    SetMajor,
}

pub async fn general_commands_handler(
    db: Database,
    bot: Bot,
    msg: Message,
    cmd: GeneralCommand,
) -> Result<()> {
    match cmd {
        GeneralCommand::Help => {
            help_command_handler(&bot, &msg).await?;
        }

        GeneralCommand::SetMajor => {
            set_major_command_handler(&db, &bot, &msg).await?;
        }
    }

    Ok(())
}

async fn help_command_handler(bot: &Bot, msg: &Message) -> Result<()> {
    let s = format!(
        "{}\n\n{}",
        GeneralCommand::descriptions(),
        TimetableCommand::descriptions()
    );

    bot.send_message(msg.chat.id, s)
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}

async fn make_majors_keyboard(db: &Database) -> Result<InlineKeyboardMarkup> {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];

    let sql = r#"SELECT * FROM majors;"#;
    let majors = sqlx::query_as::<_, MajorEntry>(sql)
        .fetch_all(db.pool.as_ref())
        .await?;

    for major in majors.chunks(3) {
        let row = major
            .iter()
            .map(|major| {
                let data = format!("{}:{}", ButtonPrefix::SetMajor, major.id);
                InlineKeyboardButton::callback(&major.title, data)
            })
            .collect();

        keyboard.push(row);
    }

    Ok(InlineKeyboardMarkup::new(keyboard))
}
pub async fn set_major_command_handler(db: &Database, bot: &Bot, msg: &Message) -> Result<()> {
    let kb = make_majors_keyboard(db).await?;

    bot.send_message(msg.chat.id, "Выберите свою группу")
        .reply_markup(kb)
        .await?;

    Ok(())
}

pub async fn set_major_callback_handler(db: Database, bot: Bot, q: CallbackQuery) -> Result<()> {
    let Some(data) = q.data else {
        return Ok(())
    };

    let Some((button_prefix, new_major_id)) = data.split_once(':') else {
        return Ok(())
    };

    if button_prefix != format!("{}", ButtonPrefix::SetMajor) {
        return Ok(());
    };

    bot.answer_callback_query(q.id).await?;

    let user_id = i64::try_from(q.from.id.0)?;

    let query = r#"INSERT INTO users (id, major_id) VALUES($1, $2) ON CONFLICT (id) DO UPDATE SET major_id = $2 RETURNING *;"#;
    let user_entry = sqlx::query_as::<_, UserEntry>(query)
        .bind(user_id)
        .bind(new_major_id)
        .fetch_one(db.pool.as_ref())
        .await?;

    let query = r#"SELECT * FROM majors WHERE id = $1;"#;
    let major_entry = sqlx::query_as::<_, MajorEntry>(query)
        .bind(&user_entry.major_id)
        .fetch_one(db.pool.as_ref())
        .await?;

    let text = format!("Вы успешно сменили группу на <b>{}</b>!", major_entry.title);

    if let Some(Message { id, chat, .. }) = q.message {
        bot.edit_message_text(chat.id, id, text)
            .parse_mode(ParseMode::Html)
            .await?;
    } else if let Some(id) = q.inline_message_id {
        bot.edit_message_text_inline(id, text)
            .parse_mode(ParseMode::Html)
            .await?;
    }

    Ok(())
}
