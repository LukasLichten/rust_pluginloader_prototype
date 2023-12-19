//! Serves to store ideas that didn't work out


/// The Manager is a failed attempt to reduce the overhead from dyn<br>
/// dyn turns a reference into the reference to the data and a vtable with the functions<br>
/// It was intended to be put into the plugin as a static mut, where the loader replaces it before calling any functions.<br>
/// However even none mutable calls inside the plugin require unsafe (Awkward).<br>
/// Also the loader needs to put the Datastore as a Static reference so it can be called by none Object bound functions.<br>
/// And as Datastore likely uses things like HashMap it is impossible to create a static of it (which requires const functions). <br>
/// <br>
/// But isn't a struct that contains function pointers effectively a vtable with extra steps?<br>
/// And we are stuck with such a design that we need to pass functions into the plugin, as the loader is the one to implement them.
#[derive(Debug, Clone)]
pub struct Manager {
    get_value: fn(key: String) -> Option<String>,
    set_value: fn(key: String, val: String),
}

impl Manager {
    /// To be used by the loader to create a manager for the plugins, not actual plugins.<br>
    /// This collects function pointers so plugins can call function on the manager
    pub fn new(get_value: fn(key: String) -> Option<String>, set_value: fn(key: String, val: String)) -> Self {
        Manager { get_value, set_value }
    }

    pub fn get_value(&self, key: String) -> Option<String> {
        (self.get_value)(key)
    }

    pub fn set_value(&self, key: String, val: String) {
        (self.set_value)(key, val)
    }
}

// We could likely make macros to build the default functions

fn set_value_default(_key: String, _val: String) {
    panic!("No reference set! Creating a Manger with Default serves to create place holders!");
}

fn get_value_default(_key: String) -> Option<String> {
    panic!("No reference set! Creating a Manger with Default serves to create place holders!");
}

/// Creates a placeholder manager, mainly to be placed in the root of the library,<br>
/// where the loader replaces it at runtime.<br>
/// <br>
/// DO NOT call the functions on this default instance, they will all panic <br>
/// <br>
/// This is done as we would otherwise need Options either on the function pointers or the entire object, <br>
/// which needless would need unwraping, when it should never be at default, as the loader will replace it prior to calling any functions
pub const fn placeholder_manager() -> Manager {
    Manager { get_value: get_value_default, set_value: set_value_default }
}