#[cfg(windows)]
#[macro_use]
extern crate memoffset;
#[cfg(windows)]
#[macro_use]
extern crate winapi;
extern crate vst;

// use cocoa::appkit::{
//     NSApp, NSApplication, NSApplicationActivateIgnoringOtherApps,
//     NSApplicationActivationPolicyRegular, NSBackingStoreBuffered, NSMenu, NSMenuItem,
//     NSRunningApplication, NSWindow, NSWindowStyleMask,
// };
// use cocoa::base::{nil, selector, NO};
// use cocoa::foundation::{NSAutoreleasePool, NSPoint, NSProcessInfo, NSRect, NSSize, NSString};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use vst::api::Supported;
use vst::host::{Host, PluginLoader};
use vst::plugin::CanDo;
use vst::plugin::Plugin;
use std::error::Error;
use std::os::raw::c_void;
use std::ptr;
use std::mem;
use winapi::*;

use core::mem::MaybeUninit;
use core::panic::PanicInfo;
use winapi::shared::minwindef::{
    LRESULT,
    LPARAM,
    LPVOID,
    WPARAM,
    UINT,
};

use winapi::shared::windef::{
    HWND,
    HMENU,
    HICON,
    HBRUSH,
};

use winapi::um::libloaderapi::GetModuleHandleA;

use winapi::um::winuser::{
    DrawTextA,
    BeginPaint,
    EndPaint,
    GetClientRect,
    DefWindowProcA,
    RegisterClassA,
    CreateWindowExA,
    TranslateMessage,
    DispatchMessageA,
    GetMessageA,
    PostQuitMessage
};

use winapi::um::winuser::{
    WNDCLASSA,
    CS_OWNDC,
    CS_HREDRAW,
    CS_VREDRAW,
    CW_USEDEFAULT,
    WS_OVERLAPPEDWINDOW,
    WS_VISIBLE,
    DT_SINGLELINE,
    DT_CENTER,
    DT_VCENTER
};

// // #[cfg(windows)]
// // mod win32;
// pub mod win32;

mod lib {
    use std::error::Error;
    use std::os::raw::c_void;

    pub type JavascriptCallback = Box<dyn Fn(String) -> String>;

    pub trait PluginGui {
        fn size(&self) -> (i32, i32);
        fn position(&self) -> (i32, i32);
        fn close(&mut self);
        fn open(&mut self, parent_handle: *mut c_void) -> bool;
        fn is_open(&mut self) -> bool;
        fn execute(&self, javascript_code: &str) -> Result<(), Box<dyn Error>>;
    }
}

pub struct PluginGui {
    gui: Box<dyn lib::PluginGui>,
}

impl PluginGui {
    // Calls the Javascript 'eval' function with the specified argument.
    // This method always returns an error when the plugin window is closed.
    pub fn execute(&self, javascript_code: &str) -> Result<(), Box<dyn Error>> {
        self.gui.execute(javascript_code)
    }
}

impl vst::editor::Editor for PluginGui {
    fn size(&self) -> (i32, i32) {
        self.gui.size()
    }

    fn position(&self) -> (i32, i32) {
        self.gui.position()
    }

    fn close(&mut self) {
        self.gui.close()
    }

    fn open(&mut self, parent_handle: *mut c_void) -> bool {
        self.gui.open(parent_handle)
    }

    fn is_open(&mut self) -> bool {
        self.gui.is_open()
    }
}

// pub use lib::JavascriptCallback;

// pub fn new_plugin_gui(
//     html_document: String,
//     js_callback: JavascriptCallback,
//     window_size: Option<(i32, i32)>) -> PluginGui
// {
//     #[cfg(windows)]
//     {
//         PluginGui {
//             gui: crate::win32::new_plugin_gui(html_document, js_callback, window_size)
//         }
//     }
// }

struct SampleHost;

impl Host for SampleHost {
    fn automate(&self, index: i32, value: f32) {
        println!("Parameter {} had its value changed to {}", index, value);
    }
}

fn main() {
    let host = Arc::new(Mutex::new(SampleHost));
    // let path =
    //     Path::new("/Library/Audio/Plug-Ins/VST/Addictive Keys.vst/Contents/MacOS/Addictive Keys");
    let path =
        Path::new("C:\\\\Program Files\\Steinberg\\VSTPlugins\\XLN Audio\\Addictive Keys.dll");

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
    // let nullptr: *mut core::ffi::c_void = std::ptr::null_mut();
    // let success = editor.open(nullptr);
    let window_ptr = open_window();
    let success = editor.open(window_ptr);
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

pub fn open_window() -> *mut core::ffi::c_void {
    // winapi::shared::windef::HWND(0)
    // let mut window = crate::win32::gui::Window::new(std::ptr::null_mut(), None);
    // &mut window.handle as *mut _ as *mut _
    let mut hwnd = create_window();
    &mut hwnd as *mut _ as *mut _
}

pub unsafe extern "system" fn window_proc(hwnd: HWND,
    msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {

    match msg {
        winapi::um::winuser::WM_PAINT => {
            let mut paint_struct = MaybeUninit::uninit();
            let mut rect = MaybeUninit::uninit();
            let hdc = BeginPaint(hwnd, paint_struct.as_mut_ptr());
            GetClientRect(hwnd, rect.as_mut_ptr());
            DrawTextA(hdc, "Hello world\0".as_ptr() as *const i8, -1, rect.as_mut_ptr(), DT_SINGLELINE | DT_CENTER | DT_VCENTER);
            EndPaint(hwnd, paint_struct.as_mut_ptr());
        }
        winapi::um::winuser::WM_DESTROY => {
            PostQuitMessage(0);
        }
        _ => { return DefWindowProcA(hwnd, msg, wparam, lparam); }
    }
    return 0;
}


fn create_window( ) -> HWND {
    unsafe {
        let hinstance = GetModuleHandleA( 0 as *const i8 );
        let wnd_class = WNDCLASSA {
            style : CS_OWNDC | CS_HREDRAW | CS_VREDRAW,     
            lpfnWndProc : Some( window_proc ),
            hInstance : hinstance,
            lpszClassName : "MyClass\0".as_ptr() as *const i8,
            cbClsExtra : 0,									
            cbWndExtra : 0,
            hIcon: 0 as HICON,
            hCursor: 0 as HICON,
            hbrBackground: 0 as HBRUSH,
            lpszMenuName: 0 as *const i8,
        };
        RegisterClassA( &wnd_class );

        CreateWindowExA(
            0,									// dwExStyle 
            "MyClass\0".as_ptr() as *const i8,		                // class we registered.
            "MiniWIN\0".as_ptr() as *const i8,						// title
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,	// dwStyle
            CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT,	// size and position
            0 as HWND,               	// hWndParent
            0 as HMENU,					// hMenu
            hinstance,                  // hInstance
            0 as LPVOID )				// lpParam
    }
}

// fn create_window( name : &str, title : &str ) -> Result {
//     let name = winapi::win32_string( name );
//     let title = winapi::win32_string( title );

//     unsafe {
//         let hinstance = GetModuleHandleW( null_mut() );
//         let wnd_class = WNDCLASSW {
//             style : CS_OWNDC | CS_HREDRAW | CS_VREDRAW,
//             lpfnWndProc : Some( DefWindowProcW ),
//             hInstance : hinstance,
//             lpszClassName : name.as_ptr(),
//             cbClsExtra : 0,
//             cbWndExtra : 0,
//             hIcon: null_mut(),
//             hCursor: null_mut(),
//             hbrBackground: null_mut(),
//             lpszMenuName: null_mut(),
//         };

//         RegisterClassW( &amp;wnd_class );

//         let handle = CreateWindowExW(
//             0,
//             name.as_ptr(),
//             title.as_ptr(),
//             WS_OVERLAPPEDWINDOW | WS_VISIBLE,
//             CW_USEDEFAULT,
//             CW_USEDEFAULT,
//             CW_USEDEFAULT,
//             CW_USEDEFAULT,
//             null_mut(),
//             null_mut(),
//             hinstance,
//             null_mut() );

//         if handle.is_null() {
//             Err( winapi::Error::last_os_error() )
//         } else {
//             Ok( Window { handle } )
//         }
//     }
// }

// pub fn open_window() -> *mut core::ffi::c_void {
//     unsafe {
//         let _pool = NSAutoreleasePool::new(nil);

//         let app = NSApp();
//         app.setActivationPolicy_(NSApplicationActivationPolicyRegular);

//         // create Menu Bar
//         let menubar = NSMenu::new(nil).autorelease();
//         let app_menu_item = NSMenuItem::new(nil).autorelease();
//         menubar.addItem_(app_menu_item);
//         app.setMainMenu_(menubar);

//         // create Application menu
//         let app_menu = NSMenu::new(nil).autorelease();
//         let quit_prefix = NSString::alloc(nil).init_str("Quit ");
//         let quit_title =
//             quit_prefix.stringByAppendingString_(NSProcessInfo::processInfo(nil).processName());
//         let quit_action = selector("terminate:");
//         let quit_key = NSString::alloc(nil).init_str("q");
//         let quit_item = NSMenuItem::alloc(nil)
//             .initWithTitle_action_keyEquivalent_(quit_title, quit_action, quit_key)
//             .autorelease();
//         app_menu.addItem_(quit_item);
//         app_menu_item.setSubmenu_(app_menu);

//         // create Window
//         let window = NSWindow::alloc(nil)
//             .initWithContentRect_styleMask_backing_defer_(
//                 NSRect::new(NSPoint::new(0., 0.), NSSize::new(200., 200.)),
//                 NSWindowStyleMask::NSTitledWindowMask,
//                 NSBackingStoreBuffered,
//                 NO,
//             )
//             .autorelease();
//         window.cascadeTopLeftFromPoint_(NSPoint::new(20., 20.));
//         window.center();
//         let title = NSString::alloc(nil).init_str("Hello World!");
//         window.setTitle_(title);
//         window.makeKeyAndOrderFront_(nil);
//         let current_app = NSRunningApplication::currentApplication(nil);
//         current_app.activateWithOptions_(NSApplicationActivateIgnoringOtherApps);
//         app.run();
//         // return &mut window as *mut _ as *mut _
//         // return &mut window.contentView() as *mut _ as *mut _
//         return &mut window.contentView() as *mut _ as *mut _
//     }
// }
