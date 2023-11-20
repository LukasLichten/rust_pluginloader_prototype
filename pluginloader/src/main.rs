use std::{sync::RwLock, collections::HashMap};

use plugin_sdk::Datastore;

fn main() {
    let data = Data { store: RwLock::new(HashMap::<String,String>::new()) };
    data.set_value("Test".to_string(), "Hello World!".to_string());

    println!("{}", data.get_value("Test".to_string()).unwrap());
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