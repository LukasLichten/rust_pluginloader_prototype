use plugin_sdk::Datastore;

/// As the Datastore reference has a static runtime we can store it like this <br>
/// This allows other functions and callbacks that don't have the reference <br>
/// (like a callback is triggered from somewhere else) to call and update the Datastore anyway <br>
/// <br>
/// However we do need to access it through an unsafe block... and it has to be an Option because it is not defined at compiletime <br>
/// <br>
/// This way could also be used for storing state within the plugin
static mut D: Option<&'static dyn Datastore> = None;

#[no_mangle]
pub fn test() {
    println!("From the test realm");

    unsafe { match D {
        Some(store) => store.set_value("Answer".to_string(), "Fuck you world".to_string()),
        None => println!("Didn't work")
    }}
}

#[no_mangle]

pub fn read_write_test(storage: &'static dyn Datastore) {
    println!("I read: {}",storage.get_value("Test".to_string()).unwrap());

    storage.set_value("Answer".to_string(), "My final message... good bye...".to_string());

    unsafe { D = Some(storage); }
}