use std::{sync::RwLock, collections::HashMap, fs, thread};

use dlopen2::wrapper::{WrapperApi, Container};
use plugin_sdk::{Datastore, InterPluginAPI};


fn main() {
    let data: &'static Data = Box::leak(Box::new(Data::new()));
    data.set_value("Test".to_string(), "Hello World!".to_string());

    let mut plugins = vec![];

    if let Ok(mut res) = fs::read_dir("lib") {
        while let Some(Ok(item)) = res.next() {
            println!("Found plugin {}", item.file_name().to_str().unwrap());

            match unsafe { Container::<Plugin>::load(item.path().as_os_str()) } {
                Ok(cont) => {
                    cont.init(data);
                    plugins.push(cont);
                },
                Err(e) => println!("Failed to load plugin {}", e)
            }
        }
    }

    let mut threads = vec![];

    for p in plugins {
        threads.push(thread::spawn(move | |  { p.update(data); }));
    }

    // Either way, we need to give the thread time to finish execution before we access Answer, else we race the thread and run into a None
    for handle in threads {
        handle.join().expect("One of the plugins died...");
    }
    //thread::sleep(time::Duration::from_millis(10));

    println!("{}", data.get_value("Answer".to_string()).unwrap());
}

struct Data {
    store: RwLock<HashMap<String, String>>,
    apis: RwLock<HashMap<String, InterPluginAPI>>
}

impl Datastore for Data {
    fn set_value(&self, key: String, val: String) {
        if let Ok(mut l) = self.store.write() {
            l.insert(key, val);
        }
    }

    fn get_value(&self, key: String) -> Option<String> {
        if let Ok(r) = self.store.read() {
            if let Some(v) = r.get(&key) {
                return Some(v.clone());
            }
        }

        None
    }

    fn set_interapi(&self, key: &str, api: plugin_sdk::InterPluginAPI) {
        if let Ok(mut l) = self.apis.write() {
            l.insert(key.to_string(), api);
        }
    }

    fn get_interapi(&self, key: &str) -> Option<plugin_sdk::InterPluginAPI> {
        if let Ok(r) = self.apis.read() {
            if let Some(v) = r.get(&(key.to_string())) {
                return Some(v.clone());
            }
        }

        None
    }
}

impl Data {
    fn new() -> Data {
        Data {store: RwLock::new(HashMap::<String,String>::new()), apis: RwLock::new(HashMap::<String,InterPluginAPI>::new())}
    }
}

#[derive(WrapperApi)]
struct Plugin {
    init: fn(storage: &'static dyn Datastore),
    update: fn(storage: &'static dyn Datastore)
}