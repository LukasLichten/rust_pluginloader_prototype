use std::{sync::{RwLock, atomic::{AtomicI64, Ordering, AtomicU64, AtomicBool}, Mutex, Arc}, collections::HashMap, fs, thread};

use dlopen2::wrapper::{WrapperApi, Container};
use plugin_sdk::{Datastore, Plugin, Value};


fn main() {
    let data: &'static Data = Box::leak(Box::new(Data::new()));
    data.create_value("Test".to_string(), Value::Str("Hello World!".to_string())).unwrap();

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

    println!("{}", data.get_value(&"Answer".to_string()).unwrap().to_string());

    // Cleaning out the plugins
    for p in plugins.iter() {
        p.end(data);
    }
}

struct Data {
    key_map: RwLock<HashMap<String, usize>>,
    plugins: RwLock<HashMap<String, InteralPlugin>>,
    data_store: RwLock<Vec<DataContainer>>
}

impl Datastore for Data {
    fn create_value(&self, key: String, val_type: Value) -> Result<(),()> {
        if let (Ok(mut map),Ok(mut store)) = (self.key_map.write(), self.data_store.write()) {
            if map.contains_key(&key) {
                return Err(());
            }

            map.insert(key.clone(), store.len());
            store.push(DataContainer { name: key, value: ValueStore::from(val_type) });


            Ok(())
        } else {
            Err(())
        }
    }

    fn set_value(&self, key: String, val: Value) -> Result<(),()> {
        if let (Ok(map),Ok(store)) = (self.key_map.read(), self.data_store.read()) {
            let index = if let Some(index) = map.get(&key) {
                index.clone()
            } else {
                return Err(());
            };
            drop(map);

            let cont = &store[index];
            if &cont.name != &key {
                return Err(());
            }
            return cont.value.update(val);
        } else {
            Err(())
        }
    }

    fn get_value(&self, key: &String) -> Result<Value,()> {
        if let (Ok(map),Ok(store)) = (self.key_map.read(), self.data_store.read()) {
            let index = if let Some(index) = map.get(key) {
                index.clone()
            } else {
                return Err(());
            };
            drop(map);

            let cont = &store[index];
            if &cont.name != key {
                return Err(());
            }
            let val = cont.value.read();
            drop(store);

            Ok(val)
        } else {
            Err(())
        }
    }

    fn register_plugin(&self, plugin: Plugin) -> Option<String> {
        let mut l = self.plugins.write().expect("Unable to write to plugin list");
        if l.contains_key(&plugin.name) {
            return None;
        }

        let access_token = plugin.name.clone() + "your mum"; // TODO implement a secure token system
        l.insert(plugin.name.clone(), InteralPlugin { plugin, access_token: access_token.clone(), switchoff_handle: Arc::new(AtomicBool::new(false)) });
        Some(access_token)
    }

    fn get_plugin(&self, name: &String) -> Option<Plugin> {
        let r = self.plugins.read().expect("Unable to read plugin list");
        if let Some(plugin) = r.get(name) {
            return Some(plugin.plugin.renew(plugin.switchoff_handle.clone()));
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
            if let Some(con) = l.remove(&index) {
                // Interacting with the plugin is no longer possible
                con.switchoff_handle.store(true, Ordering::Release); 
            }
        }

        false
    }
}

impl Data {
    fn new() -> Data {
        Data {key_map: RwLock::new(HashMap::<String,usize>::new()), plugins: RwLock::new(HashMap::<String,InteralPlugin>::new()), data_store: RwLock::new(vec![])}
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
    access_token: String,
    switchoff_handle: Arc<AtomicBool>
}

struct DataContainer {
    name: String,
    value: ValueStore
}

pub enum ValueStore {
    Int(AtomicI64),
    Float(AtomicU64),
    Bool(AtomicBool),
    Str(Mutex<String>)
}

impl From<Value> for ValueStore {
    fn from(value: Value) -> Self {
        match value {
            Value::Int(i) => ValueStore::Int(AtomicI64::new(i)),
            Value::Float(f) => ValueStore::Float(AtomicU64::new(u64::from_be_bytes(f.to_be_bytes()))),
            Value::Bool(b) => ValueStore::Bool(AtomicBool::new(b)),
            Value::Str(str) => ValueStore::Str(Mutex::new(str)),
        }
    }
}

impl ValueStore {
    pub fn update(&self, value: Value) -> Result<(), ()> {
        match self {
            ValueStore::Int(i) => i.store(value.try_into().ok().ok_or(())?, Ordering::Relaxed),
            ValueStore::Float(f) => {
                let float:f64 = value.try_into().ok().ok_or(())?;

                let by = float.to_be_bytes();
                let u = u64::from_be_bytes(by);
                f.store(u, Ordering::Relaxed);
            },
            ValueStore::Bool(b) => b.store(value.try_into().ok().ok_or(())?, Ordering::Relaxed),
            ValueStore::Str(str) => *str.lock().unwrap() = value.into(),
        }

        Ok(())
    }

    pub fn read(&self) -> Value {
        match self {
            ValueStore::Int(i) => Value::Int(i.load(Ordering::Relaxed)),
            ValueStore::Float(f) => {
                let u = f.load(Ordering::Relaxed);
                Value::Float(f64::from_be_bytes(u.to_be_bytes()))
            },
            ValueStore::Bool(b) => Value::Bool(b.load(Ordering::Relaxed)),
            ValueStore::Str(str) => Value::Str(str.lock().unwrap().clone()),
        }
    }
}