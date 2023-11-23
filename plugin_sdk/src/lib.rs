pub mod depreciated;

pub trait Datastore: Sync {
    fn set_value(&self, key: String, val: String);

    fn get_value(&self, key: &String) -> Option<String>;

    fn register_plugin(&self, plugin: Plugin) -> Option<String>;

    fn deregister_plugin(&self, access_token: &String) -> bool;

    fn get_plugin(&self, name: &String) -> Option<Plugin>;
}

#[derive(Debug, Clone)]
pub struct Plugin {
    run: Option<fn(methode: String, args: String) -> Result<String, ()>>,
    pub version: String,
    pub name: String
}

impl Plugin {
    pub fn new(name: String, version: String, interact_func: Option<fn(methode: String, args: String) -> Result<String, ()>>) -> Self {
        Plugin { run: interact_func, version, name }
    }

    pub fn interact(&self, methode: String, args: String) -> Result<String, ()> {
        if let Some(met) = self.run {
            return met(methode, args);
        }

        Err(())
    }
}