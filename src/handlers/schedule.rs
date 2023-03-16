use anyhow::{bail, Result};
use chrono::{DateTime, Datelike, FixedOffset};
use teloxide::{
    prelude::*,
    types::{Chat, MessageId, ParseMode},
    Bot,
};

use crate::utils::{
    database::Database,
    sql::types::{DayOfWeek, TimeTableEntry, UserEntry, WeekType},
};

async fn get_user(user_id: &UserId, db: &Database) -> Result<Option<UserEntry>> {
    let id = i64::try_from(user_id.0).unwrap();

    let result = sqlx::query_as::<_, UserEntry>(r#"SELECT * FROM users WHERE id = $1;"#)
        .bind(id)
        .fetch_optional(db.pool.as_ref())
        .await?;

    Ok(result)
}

pub async fn filter_predicate(bot: &Bot, msg: &Message, db: &Database) -> Result<bool> {
    let Some(user) = msg.from() else {
        bail!("Объект пользователя не связан с сообщением.")
    };

    let Some(_) = get_user(&user.id, db).await? else {
        bot.send_message(msg.chat.id, "Вы должны указать свою группу!\nИспользуйте /setmajor").await?;
        return Ok(false)
    };

    Ok(true)
}

fn format_entry(entry: &TimeTableEntry) -> Result<String> {
    let mut s = format!(
        "{} – {}",
        entry.starts_at.format("%H:%M"),
        entry.ends_at.format("%H:%M")
    );

    s = format!("{s}\n<b>{}</b>", entry.subject_name);
    s = format!("{s}\n    {}", entry.subject_type);
    if let Some(value) = entry.professor.as_ref() {
        s = format!("{s}\n    {value}");
    }
    s = format!("{s}\n    {}", entry.auditorium);

    Ok(s)
}

fn format_entries(entries: &[TimeTableEntry], dt: &DateTime<FixedOffset>) -> Result<String> {
    let mut s = String::new();

    entries.iter().for_each(|entry| {
        let formatted = format_entry(entry).unwrap();

        if !s.is_empty() {
            s = format!("{s}\n\n{formatted}");
        } else {
            s = formatted;
        }
    });

    s = format!(
        "<b>Расписание занятий на {}</b>\n\n\n{s}",
        dt.format_localized("%e %B", chrono::Locale::ru_RU)
            .to_string()
            .trim()
    );

    Ok(s)
}

async fn find_timetable(db: &Database, dt: &DateTime<FixedOffset>, major_id: &String) -> Result<Vec<TimeTableEntry>> {
    let day_of_week: DayOfWeek = dt.weekday().into();
    let week: WeekType = (*dt).into(); // TODO: might something stupid

    let entries = sqlx::query_as::<_, TimeTableEntry>(
        r#"SELECT * FROM public.timetable 
        WHERE 
            week = $1 
            AND day_of_week = $2
            AND major_id = $3 
        ORDER BY starts_at;"#,
    )
    .bind(&week)
    .bind(&day_of_week)
    .bind(major_id)
    .fetch_all(db.pool.as_ref())
    .await?;

    Ok(entries)
}

async fn prepate_text(db: &Database, dt: &DateTime<FixedOffset>, major_id: &String) -> Result<String> {
    let entries = find_timetable(db, dt, major_id).await?;
    let text = if !entries.is_empty() {
        format_entries(&entries, dt)?
    } else {
        "<i>Ничего не найдено.</i>".to_owned()
    };

    Ok(text)
}

pub async fn command_handler(
    db: &Database,
    bot: &Bot,
    dt: DateTime<FixedOffset>,
    major_id: &String,
    chat: &Chat,
) -> Result<()> {
    let text = prepate_text(db, &dt, major_id).await?;
    bot.send_message(chat.id, text)
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}

pub async fn button_handler_known_chat(
    db: &Database,
    bot: &Bot,
    dt: DateTime<FixedOffset>,
    major_id: &String,
    chat: &Chat,
    message_id: MessageId,
) -> Result<()> {
    let text = prepate_text(db, &dt, major_id).await?;
    bot.edit_message_text(chat.id, message_id, text)
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}

pub async fn button_handler_unknown_chat(
    db: &Database,
    bot: &Bot,
    dt: DateTime<FixedOffset>,
    major_id: &String,
    id: String,
) -> Result<()> {
    let text = prepate_text(db, &dt, major_id).await?;
    bot.edit_message_text_inline(id, text)
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}
