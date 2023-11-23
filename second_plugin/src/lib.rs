use std::sync::OnceLock;

use plugin_sdk::{Datastore, Plugin};

static TOKEN: OnceLock<String> = OnceLock::new();

#[no_mangle]
pub fn init(storage: &'static dyn Datastore) -> Result<(), String> {
    // Other way of creating the plugin:
    // We define the values here in place
    let p = Plugin::new("Second Plugin".to_string(), "0.1.0".to_string(), None);
    if let Some(token) = storage.register_plugin(p) {
        TOKEN.set(token).expect("Init was called twice!");


    } else {
        return Err("Second Plugin was unable to register".to_string());
    }

    storage.set_value("Plugin2".to_string(), "New Kid".to_string());

    Ok(())
}

#[no_mangle]
pub fn update(storage: &'static dyn Datastore) {
    println!("I see plugin 1 was here: {}",storage.get_value(&"Plugin1".to_string()).unwrap());

    //storage.get_plugin(&"test_plugin".to_string()).unwrap().interact("".to_string(), "".to_string()).unwrap();

    storage.set_value("Finish".to_string(), "Last laugh".to_string());
}

#[no_mangle]
pub fn end(storage: &dyn Datastore) {
    println!("{}, the downfall of western civilization", storage.deregister_plugin(TOKEN.get().unwrap()));

}
