use plugin_sdk::Datastore;

#[no_mangle]
pub fn init(storage: &'static dyn Datastore) {
    storage.set_value("Plugin2".to_string(), "New Kid".to_string());
}

#[no_mangle]
pub fn update(storage: &'static dyn Datastore) {
    println!("I see plugin 1 was here: {}",storage.get_value("Plugin1".to_string()).unwrap());

    storage.set_value("Finish".to_string(), "Last laugh".to_string());
}
