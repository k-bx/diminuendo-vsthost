#![feature(backtrace)]
use apres::{MIDIEvent, MIDI};
use chrono::prelude::*;
use midi_control::{ControlEvent, KeyEvent, MidiMessage};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::Row;
use std::backtrace::Backtrace;
use thiserror::Error;
use tokio_stream::StreamExt;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("BL")]
    BL { msg: String },
    #[error("sqlx::Error")]
    Sqlx {
        #[from]
        source: sqlx::Error,
        backtrace: Backtrace,
    },
    #[error("chrono::ParseError")]
    Chrono {
        #[from]
        source: chrono::ParseError,
        backtrace: Backtrace,
    },
}

impl From<&str> for AppError {
    fn from(v: &str) -> Self {
        AppError::BL { msg: v.to_string() }
    }
}

impl From<String> for AppError {
    fn from(v: String) -> Self {
        AppError::BL { msg: v }
    }
}

#[tokio::main]
pub async fn main() -> Result<(), AppError> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite:/home/pi/storage/diminuendo.sqlite")
        .await?;

    let mut rows = sqlx::query(
        r#"
select
  strftime("%Y-%m-%d %H:%M:00", ts/1000.0, 'unixepoch') as dt, 
  sum(length(events)) as events_len 
from events 
group by dt
order by dt desc
"#,
    )
    .fetch(&pool);
    let mut all_events: Vec<(DateTime<Utc>, i64)> = vec![];
    while let Some(row) = rows.try_next().await? {
        let dt: DateTime<Utc> = parse_minute_date(row.try_get("dt")?)?;
        let events_len: i64 = row.try_get("events_len")?;
        all_events.push((dt, events_len));
    }
    all_events.reverse();

    println!("> found {} minutes with data", all_events.len());

    let mut groups: Vec<Vec<(DateTime<Utc>, i64)>> = vec![];
    let mut last_timestamp: Option<DateTime<Utc>> = None;
    let mut group_acc: Vec<(DateTime<Utc>, i64)> = vec![];
    for event in all_events.iter() {
        let mut new_group_created = false;
        if let Some(last_timestamp) = last_timestamp {
            if event.0.signed_duration_since(last_timestamp) >= chrono::Duration::minutes(10) {
                groups.push(group_acc.clone());
                group_acc = vec![event.clone()];
                new_group_created = true;
            }
        }
        if !new_group_created {
            group_acc.push(event.clone());
        }
        last_timestamp = Some(event.0);
    }
    if group_acc.len() > 0 {
        groups.push(group_acc.clone());
    }

    println!("> grouped into {} minute groups", groups.len());
    groups.reverse();

    // for group in groups.iter() {
    //     let (begin, end) = group_begin_end(group)?;
    //     println!(
    //         "> group [{:?}, {:?}) - {} events",
    //         begin,
    //         end,
    //         group.iter().map(|x| x.1).sum::<i64>()
    //     );
    // }

    let latest_group = groups.first().ok_or("No groups found")?;
    let (latest_begin, latest_end) = group_begin_end(latest_group)?;
    println!(
        "> group [{:?}, {:?}) - {} events",
        latest_begin,
        latest_end,
        latest_group.iter().map(|x| x.1).sum::<i64>()
    );
    group_midi_to_wav(&pool, latest_begin, latest_end).await?;

    Ok(())
}

pub fn parse_minute_date(s: &str) -> Result<DateTime<Utc>, AppError> {
    Ok(DateTime::from_utc(
        chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:00")?,
        Utc,
    ))
}

/// Last minute should be treated as open, e.g. [a, b)
pub fn group_begin_end(
    group: &Vec<(DateTime<Utc>, i64)>,
) -> Result<(DateTime<Utc>, DateTime<Utc>), AppError> {
    let first = group.first().ok_or("Group with no first".to_string())?;
    let last = group.last().ok_or("Group with no last")?;
    let last_plus_minute = last.0 + chrono::Duration::minutes(1);
    Ok((first.0, last_plus_minute))
}

/// End is excluding
pub async fn group_midi_to_wav(
    pool: &SqlitePool,
    begin: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<(), AppError> {
    let begin_ts = begin.timestamp_millis();
    let mut rows =
        sqlx::query(r#"select ts, events from events where ts >= ? and ts < ? order by ts asc"#)
            .bind(begin.timestamp_millis())
            .bind(end.timestamp_millis())
            .fetch(pool);
    let mut midi = MIDI::new();
    println!("> get_ppqn: {}", midi.get_ppqn());
    let mut first_ts = None;
    while let Some(row) = rows.try_next().await? {
        let ts: i64 = row.try_get("ts")?;
        if let None = first_ts {
            first_ts = Some(ts);
        }
        let events: Vec<u8> = row.try_get("events")?;
        let data: &[u8] = &events;
        // println!("> data: {:?}", &data);
        let messages: Vec<MidiMessage> = midi_control::message::from_multi_filtered(&data).unwrap();
        // println!("> messages: {:?}", messages);
        for msg in messages.iter() {
            let track = 0;
            let wait = (ts - first_ts.unwrap_or(begin_ts)) * 240 / 1000;
            match msg {
                MidiMessage::NoteOn(_ch, KeyEvent { key, value }) => {
                    let event = MIDIEvent::NoteOn(0, *key, *value);
                    midi.insert_event(track, wait as usize, event);
                }
                MidiMessage::NoteOff(_ch, KeyEvent { key, value }) => {
                    let event = MIDIEvent::NoteOff(0, *key, *value);
                    midi.insert_event(track, wait as usize, event);
                }
                MidiMessage::ControlChange(ch, ControlEvent { control, value }) => {
                    let event = MIDIEvent::ControlChange(*ch as u8, *control, *value);
                    midi.insert_event(track, wait as usize, event);
                }
                _ => {}
            }
        }
        // midi.push_event(0, 1800, MIDIEvent::NoteOff(0, 64, 100));
    }

    // // midi.insert_event(0, 0, MIDIEvent::NoteOn(0, 64, 100));
    // // midi.push_event(0, 120, MIDIEvent::NoteOn(0, 64, 100));
    midi.save("target/output.mid");
    Ok(())
}

pub fn from_timestamp_millis(millis: i64) -> DateTime<Utc> {
    DateTime::from_utc(
        NaiveDateTime::from_timestamp(millis / 1000, (millis % 1000) as u32),
        Utc,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minute_date() {
        let dt: DateTime<Utc> = parse_minute_date("2021-06-06 15:31:00").unwrap();
        println!("> dt: {:?}", dt);
    }
}
