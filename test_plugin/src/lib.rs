use std::{sync::{Mutex, OnceLock}, thread::{JoinHandle, self}};

use plugin_sdk::{Datastore, Plugin, Value};

static RUNNER: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);

static STATE: OnceLock<State> = OnceLock::new();

mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[no_mangle]
pub fn init(storage: &'static dyn Datastore) -> Result<(), String> {
    // This is one way of creating the plugin data:
    // Using the built crate at compile time (with the build.rs, and then importing it via built_info)
    // This allows us to update the version and name directly from the Cargo.toml
    let p = Plugin::new(built_info::PKG_NAME.to_string(), built_info::PKG_VERSION.to_string(), Some(run));
    
    if let Some(token) = storage.register_plugin(p) {
        STATE.set(State {
            access_token: token,
            test_value: Mutex::new(3)
        }).expect("Init should never be called twice");
    } else {
        // It is important to error out if the plugin failed to initalize, as you would risk errors when the program ends
        // Also it is more professional for the plugin to return an error rather then panicing and taking the program with it
        return Err(format!("Unable to Register Plugin {} v{}",built_info::PKG_NAME, built_info::PKG_VERSION));
    }
    
    
    storage.create_value("Plugin1".to_string(), Value::Str("Fuck you world".to_string())).unwrap();
    

    *RUNNER.lock().unwrap() = Some(thread::spawn(move | | { update(storage); }));
    Ok(())
}

// Update function is optional, so we can run it in our own thread
// #[no_mangle]
fn update(storage: &'static dyn Datastore) {
    println!("I read: {}",storage.get_value(&"Test".to_string()).unwrap().to_string());

    let mut index = 0;
    let start = std::time::Instant::now();
    while let Err(()) = storage.get_value(&"Finish".to_string()) {
        index += 1;
    }
    println!("So Plugin 2 updated after {}ns and {} iter: {}",start.elapsed().as_nanos(), index, storage.get_value(&"Finish".to_string()).unwrap().to_string());

    storage.create_value("Answer".to_string(), Value::Str("My final message... good bye...".to_string())).unwrap();
}

pub fn run(_methode: String, _args: String) -> Result<String, ()> {
    test();
    Ok("".to_string())
}

fn test() {
    *STATE.get().unwrap().test_value.lock().unwrap() = 5;
    println!("Around the world...");
}

#[no_mangle]
pub fn end(storage: &dyn Datastore) {
    if let Some(handle) = (*RUNNER.lock().unwrap()).take() {
        handle.join().expect("Thread failed... sad owo");
    }
    
    
    println!("It is joe-over! {} {}", STATE.get().unwrap().test_value.lock().unwrap(), storage.deregister_plugin(&STATE.get().unwrap().access_token));
}

#[derive(Debug)]
struct State {
    access_token: String,
    test_value: Mutex<i32>
}