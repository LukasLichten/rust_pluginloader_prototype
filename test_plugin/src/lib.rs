use plugin_sdk::Datastore;


#[no_mangle]
pub fn test() {
    println!("From the test realm")
}

#[no_mangle]

pub fn read_write_test(storage: &dyn Datastore) {
    println!("I read: {}",storage.get_value("Test".to_string()).unwrap());

    storage.set_value("Answer".to_string(), "My final message... good bye...".to_string());
}