extern crate libloading;
use std::env;
use libloading::{Library, Symbol};
use fetish_lib::everything::*;

type GenerateContextFunc = unsafe fn(&str) -> serde_json::Result<String>;
type DeserializeContextFunc = unsafe fn(&str) -> serde_json::Result<Context>;

pub struct ContextDefinitionLibraryHandle<'a> {
    generate_context_symbol : Symbol<'a, GenerateContextFunc>,
    deserialize_context_symbol : Symbol<'a, DeserializeContextFunc>
}

impl <'a> ContextDefinitionLibraryHandle<'a> {
    pub fn new(lib : &'a Library) -> Result<ContextDefinitionLibraryHandle<'a>, libloading::Error> {
        unsafe {
            let generate_context_symbol : Symbol<GenerateContextFunc> = lib.get(b"generate_context_json")?;
            let deserialize_context_symbol : Symbol<DeserializeContextFunc> = lib.get(b"deserialize_context_json")?;
            Result::Ok(ContextDefinitionLibraryHandle {
                generate_context_symbol,
                deserialize_context_symbol
            })
        }
    }
    pub fn generate_context_json(&self, param_json : &str) -> Result<String, serde_json::Error> {
        unsafe {
            let generate_context = &self.generate_context_symbol;
            generate_context(param_json)
        }
    }
    pub fn deserialize_context_json(&self, context_json : &str) -> Result<Context, serde_json::Error> {
        unsafe {
            let deserialize_context = &self.deserialize_context_symbol;
            deserialize_context(context_json)
        }
    }
}
