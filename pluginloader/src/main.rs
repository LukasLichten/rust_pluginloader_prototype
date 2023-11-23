use std::{sync::RwLock, collections::HashMap, fs, thread};

use dlopen2::wrapper::{WrapperApi, Container};
use plugin_sdk::{Datastore, Plugin};


fn main() {
    let data: &'static Data = Box::leak(Box::new(Data::new()));
    data.set_value("Test".to_string(), "Hello World!".to_string());

    let plugins: &'static mut Vec<Container<PluginWrapper>> = Box::leak(Box::new(vec![]));

    if let Ok(mut res) = fs::read_dir("lib") {
        while let Some(Ok(item)) = res.next() {
            println!("Found plugin {}", item.file_name().to_str().unwrap());

            match unsafe { Container::<PluginWrapper>::load(item.path().as_os_str()) } {
                Ok(cont) => {
                    match cont.init(data) {
                        Ok(()) => plugins.push(cont),
                        Err(e) => println!("Failed to load plugin {}", e)
                    };
                },
                Err(e) => println!("Failed to load plugin {}", e)
            }
        }
    }

    let mut threads = vec![];

    for p in plugins.iter() {
        if p.has_update() {
            threads.push(thread::spawn(move | |  { p.update(data); }));
        }
        
    }

    // Either way, we need to give the thread time to finish execution before we access Answer, else we race the thread and run into a None
    for handle in threads {
        handle.join().expect("One of the plugins died...");
    }
    thread::sleep(std::time::Duration::from_millis(10));

    println!("{}", data.get_value(&"Answer".to_string()).unwrap());

    // Cleaning out the plugins
    for p in plugins.iter() {
        p.end(data);
    }
}

struct Data {
    store: RwLock<HashMap<String, String>>,
    plugins: RwLock<HashMap<String, InteralPlugin>>
}

impl Datastore for Data {
    fn set_value(&self, key: String, val: String) {
        if let Ok(mut l) = self.store.write() {
            l.insert(key, val);
        }
    }

    fn get_value(&self, key: &String) -> Option<String> {
        if let Ok(r) = self.store.read() {
            if let Some(v) = r.get(key) {
                return Some(v.clone());
            }
        }

        None
    }

    fn register_plugin(&self, plugin: Plugin) -> Option<String> {
        let mut l = self.plugins.write().expect("Unable to write to plugin list");
        if l.contains_key(&plugin.name) {
            return None;
        }

        let access_token = plugin.name.clone() + "your mum"; // TODO implement a secure token system
        l.insert(plugin.name.clone(), InteralPlugin { plugin, access_token: access_token.clone() });
        Some(access_token)
    }

    fn get_plugin(&self, name: &String) -> Option<Plugin> {
        let r = self.plugins.read().expect("Unable to read plugin list");
        if let Some(plugin) = r.get(name) {
            return Some(plugin.plugin.clone());
        }

        None
    }

    fn deregister_plugin(&self, access_token: &String) -> bool {
        let mut l = self.plugins.write().expect("Unable to write to plugin list");
        let mut index: Option<String> = None;

        for (name, plugin) in l.iter() {
            if &plugin.access_token == access_token {
                index = Some(name.clone());
            }
        }

        if let Some(index) = index {
            return l.remove(&index).is_some();
        }

        false
    }
}

impl Data {
    fn new() -> Data {
        Data {store: RwLock::new(HashMap::<String,String>::new()), plugins: RwLock::new(HashMap::<String,InteralPlugin>::new())}
    }
}

#[derive(WrapperApi)]
struct PluginWrapper {
    init: fn(storage: &'static dyn Datastore) -> Result<(), String>,
    update: Option<fn(storage: &'static dyn Datastore)>,
    end: fn(storage: &'static dyn Datastore)
}


struct InteralPlugin {
    plugin: Plugin,
    access_token: String
}