pub mod depreciated;

pub trait Datastore {
    fn set_value(&self, key: String, val: String);

    fn get_value(&self, key: String) -> Option<String>;

    fn set_interapi(&self, key: &str, api: InterPluginAPI);

    fn get_interapi(&self, key: &str) -> Option<InterPluginAPI>;
}

#[derive(Debug, Clone)]
pub struct InterPluginAPI {
    pub functions: Vec<(String, ApiFunc)>
}

#[derive(Debug, Clone)]
pub enum ApiFunc {
    Basic(fn()),
    Advanced(fn(String) -> String)
}