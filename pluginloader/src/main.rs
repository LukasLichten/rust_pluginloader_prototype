use std::{sync::{RwLock, atomic::{AtomicI64, Ordering, AtomicU64, AtomicBool}, Mutex, Arc}, collections::{HashMap, hash_map::DefaultHasher}, fs, thread, hash::Hasher};
use std::hash::Hash;

use dlopen2::wrapper::{WrapperApi, Container};
use plugin_sdk::{Datastore, Plugin, Value, AccessToken, DataHandle};


fn main() {
    let data: &'static Data = Box::leak(Box::new(Data::new()));
    // data.create_value("Test".to_string(), Value::Str("Hello World!".to_string())).unwrap();

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

    println!("{}", data.get_value(&data.get_data_handle("Answer").unwrap()).unwrap().to_string());

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
    fn create_value(&self, key: String, access_token: &AccessToken, val_type: Value) -> Result<DataHandle,()> {
        if let (Ok(mut map),Ok(mut store)) = (self.key_map.write(), self.data_store.write()) {
            // We should prepend the namespace for this specific plugin...
            // or not, too complicated for this prototype
            if map.contains_key(&key) {
                return Err(());
            }

            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);


            let handle = DataHandle { index: store.len(), name_hash: hasher.finish() };

            map.insert(key.clone(), handle.index.clone());
            store.push(DataContainer { name: key, value: ValueStore::from(val_type), owner: access_token.clone(), name_hash: handle.name_hash.clone() });


            Ok(handle)
        } else {
            Err(())
        }
    }

    fn set_value(&self, handle: &DataHandle, access_token: &AccessToken, val: Value) -> Result<(),()> {
        if let Ok(store) = self.data_store.read() {
            if let Some(cont) = store.get(handle.index) {
                if cont.name_hash != handle.name_hash {
                    return Err(()); // Name was updated, so the handle is outdated
                }
                
                if &cont.owner != access_token {
                    return Err(());
                    // This is not the owner, therefore does not have write permission
                }

                if cont.value.update(val).is_ok() {
                    return Ok(());
                } else {
                    return Err(());
                }
            } else {
                return Err(());
            }
        } else {
            Err(())
        }
    }

    fn get_value(&self, handle: &DataHandle) -> Result<Value,()> {
        if let Ok(store) = self.data_store.read() {
            if let Some(cont) = store.get(handle.index) {
                if cont.name_hash != handle.name_hash {
                    return Err(()); // Name was updated, so the handle is outdated
                }

                return Ok(cont.value.read());
            } else {
                return Err(());
            }
        } else {
            Err(())
        }
    }

    fn register_plugin(&self, plugin: Plugin) -> Option<AccessToken> {
        let mut l = self.plugins.write().expect("Unable to write to plugin list");
        if l.contains_key(&plugin.name) {
            return None;
        }

        let access_token = AccessToken::new(plugin.name.clone() + "your mum"); // TODO implement a secure token system
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

    fn deregister_plugin(&self, access_token: &AccessToken) -> bool {
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

        drop(l);

        // We should also unload the propertys the plugins created
        

        true
    }

    fn get_data_handle(&self, key: &str) -> Option<DataHandle> {
        if let (Ok(map),Ok(store)) = (self.key_map.read(),self.data_store.read()) {
            if let Some(addr) = map.get(key) {
                if let Some(item) = store.get(addr.clone()) {

                    return Some(DataHandle { index: addr.clone(), name_hash: item.name_hash });
                }
            }

        }

        None
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
    access_token: AccessToken,
    switchoff_handle: Arc<AtomicBool>
}

#[allow(dead_code)]
struct DataContainer {
    name: String,
    owner: AccessToken,
    name_hash: u64,
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
