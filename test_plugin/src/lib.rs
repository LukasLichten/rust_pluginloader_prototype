use plugin_sdk::{Datastore, InterPluginAPI, ApiFunc};

/// As the Datastore reference has a static runtime we can store it like this <br>
/// This allows other functions and callbacks that don't have the reference <br>
/// (like a callback is triggered from somewhere else) to call and update the Datastore anyway <br>
/// <br>
/// However we do need to access it through an unsafe block... and it has to be an Option because it is not defined at compiletime <br>
/// <br>
/// This way could also be used for storing state within the plugin
static mut D: Option<&'static dyn Datastore> = None;

#[no_mangle]
pub fn init(storage: &'static dyn Datastore) {
    unsafe { D = Some(storage); }
    //println!("From the test realm");

    storage.set_value("Plugin1".to_string(), "Fuck you world".to_string());

    let api = InterPluginAPI { functions: vec![("test".to_string(),ApiFunc::Basic(test))] };
    storage.set_interapi("plugin1", api);

    // unsafe { match D {
    //     Some(store) => store.set_value("Plugin1".to_string(), "Fuck you world".to_string()),
    //     None => println!("Didn't work")
    // }}
}

#[no_mangle]
pub fn update(storage: &'static dyn Datastore) {
    println!("I read: {}",storage.get_value("Test".to_string()).unwrap());

    let mut index = 0;
    let start = std::time::Instant::now();
    while let None = storage.get_value("Finish".to_string()) {
        index += 1;
    }
    println!("So Plugin 2 updated after {}ns and {} iter: {}",start.elapsed().as_nanos(), index, storage.get_value("Finish".to_string()).unwrap());

    storage.set_value("Answer".to_string(), "My final message... good bye...".to_string());
}

fn test() {
    println!("Around the world...");
}