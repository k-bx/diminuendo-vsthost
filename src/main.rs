use apres::{MIDIEvent, MIDI};
use midi_control::{KeyEvent, MidiMessage};

fn main() {
    let data: Vec<u8> = vec![
        0x00, 0x00, 0x09, 0x90, 0x3C, 0x3F, 0x00, 0x00, 0x09, 0x90, 0x3C, 0x00, 0x00, 0x00,
    ];

    let data: &[u8] = &data;
    let messages: Vec<MidiMessage> = midi_control::message::from_multi_filtered(&data).unwrap();
    println!("{:?}", messages);

    let mut midi = MIDI::new();
    // "C" pressed -> ["0x09", "0x90", "0x3C", "0x3F"]
    // "C" unpressed -> ["0x09", "0x90", "0x3C"]
    // midi.insert_event(0, 0, MIDIEvent::NoteOn(0, 64, 100));
    // midi.push_event(0, 120, MIDIEvent::NoteOn(0, 64, 100));
    for (i, msg) in messages.iter().enumerate() {
        let track = 0;
        let wait = i * 60; // todo compute from timestamp
        match msg {
            MidiMessage::NoteOn(_ch, KeyEvent { key, value }) => {
                let event = MIDIEvent::NoteOn(0, *key, *value);
                midi.push_event(track, wait, event);
            }
            // todo rest
            _ => {}
        }
    }
    midi.push_event(0, 1800, MIDIEvent::NoteOff(0, 64, 100));
    midi.save("target/output.mid");
}
