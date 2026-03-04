use std::collections::HashMap;

use opal_runtime::Value;

pub type NativeFunction = Box<dyn Fn(&[Value], &mut dyn std::io::Write) -> Result<Value, String>>;

pub struct PluginRegistry {
    plugins: HashMap<String, HashMap<String, NativeFunction>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    pub fn register_plugin(&mut self, name: &str, functions: HashMap<String, NativeFunction>) {
        self.plugins.insert(name.to_string(), functions);
    }

    pub fn get_function(&self, plugin: &str, name: &str) -> Option<&NativeFunction> {
        self.plugins.get(plugin)?.get(name)
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
