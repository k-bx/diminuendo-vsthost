use std::path::Path;
use std::sync::{Arc, Mutex};

use vst::host::{Host, PluginLoader};
use vst::plugin::Plugin;

struct SampleHost;

impl Host for SampleHost {
    fn automate(&self, index: i32, value: f32) {
        println!("Parameter {} had its value changed to {}", index, value);
    }
}

fn main() {
    let host = Arc::new(Mutex::new(SampleHost));
    let path = Path::new("/Library/Audio/Plug-Ins/VST/Addictive Keys.vst/Contents/MacOS/Addictive Keys");

    let mut loader = PluginLoader::load(path, host.clone()).unwrap();
    let mut instance = loader.instance().unwrap();

    println!("Loaded {}", instance.get_info().name);

    instance.init();
    println!("Initialized instance!");

    println!("Closing instance...");
}
