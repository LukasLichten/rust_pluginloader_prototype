use std::sync::OnceLock;

use plugin_sdk::{Datastore, Plugin, Value, AccessToken, DataHandle};

static TOKEN: OnceLock<AccessToken> = OnceLock::new();
static HANDLE: OnceLock<DataHandle> = OnceLock::new();

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

    HANDLE.set(storage.create_value("Finish".to_string(), TOKEN.get().unwrap(), Value::Float(0.0)).unwrap()).expect("Init called twice");

    storage.create_value("Plugin2".to_string(), TOKEN.get().unwrap(), Value::Str("New Kid".to_string())).unwrap();

    Ok(())
}

#[no_mangle]
pub fn update(storage: &'static dyn Datastore) {
    println!("I see plugin 1 was here: {}",storage.get_value(&storage.get_data_handle("Plugin1").unwrap()).unwrap().to_string());


    //storage.create_value("Finish".to_string(), Value::Str("Last laugh".to_string())).unwrap();
    storage.set_value(&HANDLE.get().unwrap(), TOKEN.get().unwrap(),  Value::Float(5.24)).unwrap();
    storage.get_plugin(&"test_plugin".to_string()).unwrap().interact("".to_string(), "".to_string()).unwrap();
}

#[no_mangle]
pub fn end(storage: &dyn Datastore) {
    println!("{}, the downfall of western civilization", storage.deregister_plugin(TOKEN.get().unwrap()));

}
