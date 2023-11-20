use std::{sync::RwLock, collections::HashMap, fs};

use dlopen2::wrapper::{WrapperApi, Container};
use plugin_sdk::Datastore;

fn main() {
    let data = Data { store: RwLock::new(HashMap::<String,String>::new()) };
    data.set_value("Test".to_string(), "Hello World!".to_string());

    println!("{}", data.get_value("Test".to_string()).unwrap());

    if let Ok(mut res) = fs::read_dir("lib") {
        while let Some(Ok(item)) = res.next() {
            println!("Found plugin {}", item.file_name().to_str().unwrap());

            match unsafe { Container::<Plugin>::load(item.path().as_os_str()) } {
                Ok(cont) => cont.test(),
                Err(e) => println!("Failed to load plugin {}", e)
            }
        }
    }
}

struct Data {
    store: RwLock<HashMap<String, String>>
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
}

#[derive(WrapperApi)]
struct Plugin {
    test: fn(),
}