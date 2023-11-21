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
Implement the functions:  
```
#[no_mangle]
pub fn test() {
    todo!();
}

#[no_mangle]
pub fn read_write_test(storeage: &'static dyn Datastore) {
    todo!();
}
```
Without no_mangle to compiler will omit the function under build.  
Variable name in the read_write_test function is irrelevant