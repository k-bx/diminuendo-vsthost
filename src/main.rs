use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use vst::api::Supported;
use vst::host::{Host, PluginLoader};
use vst::plugin::CanDo;
use vst::plugin::Plugin;

struct SampleHost;

impl Host for SampleHost {
    fn automate(&self, index: i32, value: f32) {
        println!("Parameter {} had its value changed to {}", index, value);
    }
}

fn main() {
    let host = Arc::new(Mutex::new(SampleHost));
    let path =
        Path::new("/Library/Audio/Plug-Ins/VST/Addictive Keys.vst/Contents/MacOS/Addictive Keys");

    let mut loader = PluginLoader::load(path, host.clone()).unwrap();
    let mut instance = loader.instance().unwrap();

    println!("Loaded {}", instance.get_info().name);
    println!("Info: {:?}", instance.get_info());
    println!(
        "Can do SendEvents: {:?}",
        show_supported(instance.can_do(CanDo::SendEvents))
    );
    println!(
        "Can do SendMidiEvent: {:?}",
        show_supported(instance.can_do(CanDo::SendMidiEvent))
    );
    println!(
        "Can do ReceiveEvents: {:?}",
        show_supported(instance.can_do(CanDo::ReceiveEvents))
    );
    println!(
        "Can do ReceiveMidiEvent: {:?}",
        show_supported(instance.can_do(CanDo::ReceiveMidiEvent))
    );
    println!(
        "Can do ReceiveTimeInfo: {:?}",
        show_supported(instance.can_do(CanDo::ReceiveTimeInfo))
    );
    println!(
        "Can do Offline: {:?}",
        show_supported(instance.can_do(CanDo::Offline))
    );
    println!(
        "Can do MidiProgramNames: {:?}",
        show_supported(instance.can_do(CanDo::MidiProgramNames))
    );
    println!(
        "Can do Bypass: {:?}",
        show_supported(instance.can_do(CanDo::Bypass))
    );
    println!(
        "Can do ReceiveSysExEvent: {:?}",
        show_supported(instance.can_do(CanDo::ReceiveSysExEvent))
    );
    println!(
        "Can do MidiSingleNoteTuningChange: {:?}",
        show_supported(instance.can_do(CanDo::MidiSingleNoteTuningChange))
    );
    println!(
        "Can do MidiKeyBasedInstrumentControl: {:?}",
        show_supported(instance.can_do(CanDo::MidiKeyBasedInstrumentControl))
    );
    // println!("Can do Othe: {:?}", show_supported(instance.can_do(CanDo::Other())));
    // println!("Input info: {:?}", instance.get_input_info());

    instance.init();
    instance.resume();
    println!("Initialized instance!");
    let mut editor: Box<dyn vst::editor::Editor> = instance.get_editor().unwrap();
    let nullptr: *mut core::ffi::c_void = std::ptr::null_mut();
    let success = editor.open(nullptr);
    println!(
        "Opening window success? {:?}; size: {:?}; position: {:?}",
        success,
        editor.size(),
        editor.position()
    );

    instance.resume();
    let event_midi1: *const vst::api::MidiEvent = &vst::api::MidiEvent {
        event_type: vst::api::EventType::Midi,
        byte_size: std::mem::size_of::<vst::api::MidiEvent>() as i32,
        delta_frames: 0,
        flags: 0,
        note_length: 20,
        note_offset: 0,
        midi_data: [0x09, 0x90, 0x3C],
        _midi_reserved: 0,
        detune: 0,
        note_off_velocity: 0,
        _reserved1: 0,
        _reserved2: 0,
    };
    let event_midi2: *const vst::api::MidiEvent = &vst::api::MidiEvent {
        event_type: vst::api::EventType::Midi,
        byte_size: std::mem::size_of::<vst::api::MidiEvent>() as i32,
        delta_frames: 0,
        flags: 0,
        note_length: 20,
        note_offset: 0,
        midi_data: [0x2F, 0x0f, 0xf8],
        // midi_data: [0x2F, 0, 0],
        _midi_reserved: 0,
        detune: 0,
        note_off_velocity: 0,
        _reserved1: 0,
        _reserved2: 0,
    };
    let event1: *mut vst::api::Event = unsafe { std::mem::transmute(event_midi1) };
    let event2: *mut vst::api::Event = unsafe { std::mem::transmute(event_midi2) };
    let events_inner: [*mut vst::api::Event; 2] = [event1, event2];
    let events = vst::api::Events {
        num_events: 4,
        _reserved: 0,
        events: events_inner,
    };
    instance.process_events(&events);

    println!("Sleeping now");
    std::thread::sleep(Duration::from_secs(4));

    println!("Closing instance...");
}

pub fn show_supported(x: Supported) -> String {
    match x {
        Supported::Yes => "Yes".to_string(),
        Supported::Maybe => "Maybe".to_string(),
        Supported::No => "No".to_string(),
        Supported::Custom(y) => format!("Custom({})", y),
    }
}
