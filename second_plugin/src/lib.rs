use plugin_sdk::{Datastore, ApiFunc};

#[no_mangle]
pub fn init(storage: &'static dyn Datastore) {
    storage.set_value("Plugin2".to_string(), "New Kid".to_string());
}

#[no_mangle]
pub fn update(storage: &'static dyn Datastore) {
    println!("I see plugin 1 was here: {}",storage.get_value("Plugin1".to_string()).unwrap());

    let (_name, func) = &storage.get_interapi("plugin1").unwrap().functions[0];
    match func {
        ApiFunc::Basic(f) => f(),
        _ => ()
    }

    storage.set_value("Finish".to_string(), "Last laugh".to_string());
}
