#![feature(backtrace)]
use apres::{MIDIEvent, MIDI};
use chrono::prelude::*;
use midi_control::{KeyEvent, MidiMessage};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::Row;
use std::backtrace::Backtrace;
use thiserror::Error;
use tokio_stream::StreamExt;

#[derive(Error, Debug)]
pub enum AppError {
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

    // todo: get and print data intervals

    // let data: Vec<u8> = vec![
    //     0x00, 0x00, 0x09, 0x90, 0x3C, 0x3F, 0x00, 0x00, 0x09, 0x90, 0x3C, 0x00, 0x00, 0x00,
    // ];

    // let data: &[u8] = &data;
    // let messages: Vec<MidiMessage> = midi_control::message::from_multi_filtered(&data).unwrap();
    // println!("{:?}", messages);

    // let mut midi = MIDI::new();
    // // "C" pressed -> ["0x09", "0x90", "0x3C", "0x3F"]
    // // "C" unpressed -> ["0x09", "0x90", "0x3C"]
    // // midi.insert_event(0, 0, MIDIEvent::NoteOn(0, 64, 100));
    // // midi.push_event(0, 120, MIDIEvent::NoteOn(0, 64, 100));
    // for (i, msg) in messages.iter().enumerate() {
    //     let track = 0;
    //     let wait = i * 60; // todo compute from timestamp
    //     match msg {
    //         MidiMessage::NoteOn(_ch, KeyEvent { key, value }) => {
    //             let event = MIDIEvent::NoteOn(0, *key, *value);
    //             midi.push_event(track, wait, event);
    //         }
    //         // todo rest
    //         _ => {}
    //     }
    // }
    // midi.push_event(0, 1800, MIDIEvent::NoteOff(0, 64, 100));
    // midi.save("target/output.mid");

    Ok(())
}

pub fn parse_minute_date(s: &str) -> Result<DateTime<Utc>, AppError> {
    Ok(DateTime::from_utc(
        chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:00")?,
        Utc,
    ))
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
