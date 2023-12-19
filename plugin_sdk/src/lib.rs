use std::{num::{ParseIntError, ParseFloatError}, str::ParseBoolError, sync::{Arc, atomic::{AtomicBool, Ordering}}};

use tokio::sync::{mpsc, oneshot};

pub mod depreciated;

pub trait Datastore: Sync {
    /// This creates a key and sets the value to a certain type<br>
    /// If the key already exist this function return Err<br>
    /// Any type set in this inilial setting will be type future submissions will be coerced into
    fn create_value(&self, key: String, access_token: &AccessToken, val_type: Value) -> Result<DataHandle,()>; // TODO: Implement some Error types

    /// Can be called on a value for a handle <br>
    /// Using a type that is not the same as the inital will result into it being converted,<br>
    /// which may Err and be returned by this function
    fn set_value(&self, handle: &DataHandle, access_token: &AccessToken, val: Value) -> Result<(),()>;

    /// Returns the Value for a handle<br>
    /// Will fail if the data has been renamed or removed
    fn get_value(&self, handle: &DataHandle) -> Result<Value, ()>;

    /// Returns you the datahandle for a given key, or Err if it doesn't exist
    fn get_data_handle(&self, key: &str) -> Option<DataHandle>;

    /// To be called in Init<br>
    /// Registers a plugin, returning the plugins access token on success
    fn register_plugin(&self, plugin: Plugin) -> Option<AccessToken>;

    /// To be called in End<br>
    /// Deregisters the plugin, allowing it to shut down
    fn deregister_plugin(&self, access_token: &AccessToken) -> bool;

    /// Returns the handle to a plugin, allowing you to contact it internally
    fn get_plugin(&self, name: &String) -> Option<Plugin>;

    
}

pub enum ManagerError {
    
}

#[derive(Debug, Clone)]
pub struct Plugin {
    // Something I realiced: What if a plugin deregisters, but another plugin had stored the handle from the previous callback...
    // There is a Chance to access unintialized/deintialized Data
    // So we have this Arc to allow all Copies to stay up to date and to no longer call the plugin if gone
    offline: Arc<AtomicBool>,
    
    run: Option<fn(methode: String, args: String) -> Result<String, ()>>,
    sender: Option<mpsc::Sender<(String,String,oneshot::Sender<Result<String, ()>>)>>,
    pub version: String,
    pub name: String
}

impl Plugin {
    pub fn new(name: String, version: String, interact_func: Option<fn(methode: String, args: String) -> Result<String, ()>>) -> Self {
        Plugin { run: interact_func, version, name, offline: Arc::new(AtomicBool::new(false)), sender: None }
    }

    /// Used by the DataStore to hand out new clones, but with the switch offhandle it stores internally
    pub fn renew(&self, switch_offhandle: Arc<AtomicBool>) -> Self {
        Plugin { offline: switch_offhandle, run: self.run.clone(), version: self.version.clone(), name: self.name.clone(), sender: self.sender.clone() }
    }

    pub fn interact(&self, methode: String, args: String) -> Result<String, ()> {
        if self.offline.load(Ordering::Acquire) {
            return Err(());
        } else if let Some(met) = self.run {
            return met(methode, args);
        } else if let Some(sender) = &self.sender {
            let (sx, rx) = oneshot::channel::<Result<String,()>>();
            if let Err(_) = sender.blocking_send((methode, args, sx)) {
                return Err(());
            }
            if let Ok(val) = rx.blocking_recv() {
                return val;
            } else {
                return Err(())
            }
        }


        Err(())
    }

    pub async fn send(&self, methode: String, args: String, callback: oneshot::Sender<Result<String,()>>) -> Result<(),()> {
        if !self.is_online() {
            return Err(());
        } else if let Some(sender) = &self.sender {
            if let Err(_) = sender.send((methode, args, callback)).await {
                return Err(());
            } else {
                return Ok(());
            }
        } else if let Some(met) = self.run {
            let res = met(methode, args);

            if let Err(_) = callback.send(res) {
                return Err(());
            } else {
                return Ok(());
            }
        }

        Err(())
    }

    pub fn coms_are_channel(&self) -> bool {
        self.sender.is_some() && self.is_online()
    }

    pub fn coms_are_func(&self) -> bool {
        self.run.is_some() && self.is_online()
    }

    pub fn is_online(&self) -> bool {
        !self.offline.load(Ordering::Acquire)
    }
}

#[derive(Debug, Clone,PartialEq)]
pub struct AccessToken {
    token: String // Allows in the future to change this to something better (like a fixed size u8)
}

impl AccessToken {
    pub const fn new(token: String) -> AccessToken {
        AccessToken { token }
    }
}

/// Handle with which you can request data
#[derive(Debug, Clone, PartialEq)]
pub struct DataHandle {
    pub index: usize,
    pub name_hash: u64
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