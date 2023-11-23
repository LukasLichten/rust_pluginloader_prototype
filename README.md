This is a practice project to building an application which loads Plugins that are delivered as libraries  

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