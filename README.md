This is a finished (but not complete) practice project to building an application which loads Plugins that are delivered as libraries  

## Results
Purpose was to test out loading binaries as plugins at runtime.  
However these plugins need to interact with the Datastore, which has to be within the loader/main executable.  
  
One solution explored here is to pass in the datastore as a static reference, allowing you to interact with it.  
Passing in function pointers would work similar, but is more cumborsome with similar issues.  
Issues are plently though:  
Everyone needs to import the sdk, only rust plugins are supported, etc.  
Small changes to the sdk (or a library imported by the sdk like tokio) could result in changes in how the data is layed out, resulting in unpredictable results.  
When all plugins are compiled with the loader like here it is fine, but this seems to me too unpredicable.  
  
Other idea is to have a C library expose the api.  
This library can be written in rust, also do the plugin loading too.  
You likely want to supply a sdk to abstract the C ness out of the api, but not necessary.  
  
Other things this project builds on that might not be optimal:  
- Datastore uses RwLock for it's interrior mutability
- A constant loop around update funktion (although we run it once right now). Combining it with messageing seems sensible, and calling only when needed.
- Multiple things partially implemented or not finished
  - Value does not support Data and Timespans
  - No namespaces for plugins
  - Lack of Error Types (a lot of blank Result::Err())
  - Propertys plugins own are not unloaded
  - Currently only works under Linux (but should easily work under Windows with a change of a single line)

Things that might be a decent idea
- DataHandles, which contain the index in a list, so we don't have to look up a hashmap each time, using a precalculated name hash to verify the name didn't change
- Messaging over Channels. Although this can be much improved:
  - Making the message type from String to Vec u8 to allow binary protocols
  - Expanding it to allow subscribing to propertys, and sending messages when they changed (polling would become obsolete)
  - When using a C ABI abstracting the channels away inside API functions

## Building
```
cargo run
```
will run the pluginloader  
  
To build the plugin and copy the *.so into lib run 
```
make plugin
```
just `make` will build the plugin too, and run the plugin loader to do a full test

## Developing Plugins
import the `plugin_sdk` as a dependency.  
Also set the library as a `dylib` in the `Cargo.toml`:
```
[lib]
crate_type = ["dylib"]
bench = false
```
  
Then implement the functions:  
```
#[no_mangle]
pub fn init(storeage: &'static dyn Datastore) {
    todo!();
}

//Optional
#[no_mangle]
pub fn update(storeage: &'static dyn Datastore) {
    todo!();
}

#[no_mangle]
pub fn end(storeage: &dyn Datastore) {
    todo!();
}
```
Without no_mangle to compiler will omit the function under build.  
Variable name in the functions is irrelevant  
  
You should also call `storage.register_plugin(plugin)` in your init, this gives you a token (that will be used in the future to set values).  
Also `storage.register_plugin(access_token)` at the end  
