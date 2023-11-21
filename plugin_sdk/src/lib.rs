pub mod depreciated;

pub trait Datastore {
    fn set_value(&self, key: String, val: String);

    fn get_value(&self, key: String) -> Option<String>;
}