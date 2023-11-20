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
Implement a function `pub fn test()` with the `#[no_mangle]`.  
Without no_mangle to compiler will omit the function under build.  