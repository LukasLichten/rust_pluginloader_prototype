use std::{num::{ParseIntError, ParseFloatError}, str::ParseBoolError, sync::{Arc, atomic::{AtomicBool, Ordering}}};

pub mod depreciated;

pub trait Datastore: Sync {
    /// This creates a key and sets the value to a certain type<br>
    /// If the key already exist this function return Err<br>
    /// Any type set in this inilial setting will be type future submissions will be coerced into
    fn create_value(&self, key: String, val_type: Value) -> Result<(),()>; // TODO: Implement some Error types

    /// Can be called on a value for a key <br>
    /// Using a type that is not the same as the inital will result into it being converted,<br>
    /// which may Err and be returned by this function
    fn set_value(&self, key: String, val: Value) -> Result<(),()>;

    /// Returns a Value, if it exists, else returns an error
    fn get_value(&self, key: &String) -> Result<Value, ()>;

    /// To be called in Init<br>
    /// Registers a plugin, returning the plugins access token on success
    fn register_plugin(&self, plugin: Plugin) -> Option<String>;

    /// To be called in End<br>
    /// Deregisters the plugin, allowing it to shut down
    fn deregister_plugin(&self, access_token: &String) -> bool;

    /// Returns the handle to a plugin, allowing you to contact it internally
    fn get_plugin(&self, name: &String) -> Option<Plugin>;

    
}

#[derive(Debug, Clone)]
pub struct Plugin {
    // Something I realiced: What if a plugin deregisters, but another plugin had stored the handle from the previous callback...
    // There is a Chance to access unintialized/deintialized Data
    // So we have this Arc to allow all Copies to stay up to date and to no longer call the plugin if gone
    offline: Arc<AtomicBool>,
    run: Option<fn(methode: String, args: String) -> Result<String, ()>>,
    pub version: String,
    pub name: String
}

impl Plugin {
    pub fn new(name: String, version: String, interact_func: Option<fn(methode: String, args: String) -> Result<String, ()>>) -> Self {
        Plugin { run: interact_func, version, name, offline: Arc::new(AtomicBool::new(false)) }
    }

    /// Used by the DataStore to hand out new clones, but with the switch offhandle it stores internally
    pub fn renew(&self, switch_offhandle: Arc<AtomicBool>) -> Self {
        Plugin { offline: switch_offhandle, run: self.run.clone(), version: self.version.clone(), name: self.name.clone() }
    }

    pub fn interact(&self, methode: String, args: String) -> Result<String, ()> {
        if self.offline.load(Ordering::Acquire) {
            return Err(());
        }
        if let Some(met) = self.run {
            return met(methode, args);
        }

        Err(())
    }
}

/// Universal Value Type for Data contained in the Datastore<br>
/// Can easily be converted into most types
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String)
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Value::Int(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Str(str) => str.clone(),
        }
    }
}

impl TryInto<i64> for Value {
    type Error = ParseIntError;

    fn try_into(self) -> Result<i64, Self::Error> {
        match self {
            Value::Int(i) => Ok(i),
            Value::Float(f) => Ok(f as i64),
            Value::Bool(b) => Ok(b as i64),
            Value::Str(str) => str.parse::<i64>(),
        }
    }
}

impl TryInto<f64> for Value {
    type Error = ParseFloatError;

    fn try_into(self) -> Result<f64, Self::Error> {
        match self {
            Value::Int(i) => Ok(i as f64),
            Value::Float(f) => Ok(f),
            Value::Bool(b) => Ok(b as i64 as f64),
            Value::Str(str) => str.parse::<f64>(),
        }
    }
}

impl TryInto<bool> for Value {
    type Error = ParseBoolError;

    fn try_into(self) -> Result<bool, Self::Error> {
        match self {
            Value::Int(i) => Ok(i > 0),
            Value::Float(f) => Ok(f > 0.0),
            Value::Bool(b) => Ok(b),
            Value::Str(str) => str.parse::<bool>(),
        }
    }
}

impl Into<String> for Value {

    fn into(self) -> String {
        self.to_string()
    }
}