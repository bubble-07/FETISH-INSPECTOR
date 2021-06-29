extern crate libloading;
use std::env;
use libloading::{Library, Symbol};
use fetish_lib::everything::*;

type GenerateContextFunc = unsafe fn(&[u8]) -> Result<Vec<u8>, String>;
type DeserializeContextFunc = unsafe fn(&[u8]) -> Result<Context, String>;

pub struct ContextDefinitionLibraryHandle<'a> {
    generate_context_symbol : Symbol<'a, GenerateContextFunc>,
    deserialize_context_symbol : Symbol<'a, DeserializeContextFunc>
}

impl <'a> ContextDefinitionLibraryHandle<'a> {
    pub fn new(lib : &'a Library) -> Result<ContextDefinitionLibraryHandle<'a>, libloading::Error> {
        unsafe {
            let generate_context_symbol : Symbol<GenerateContextFunc> = lib.get(b"generate_serialized_context")?;
            let deserialize_context_symbol : Symbol<DeserializeContextFunc> = lib.get(b"deserialize_serialized_context")?;
            Result::Ok(ContextDefinitionLibraryHandle {
                generate_context_symbol,
                deserialize_context_symbol
            })
        }
    }
    pub fn generate_serialized_context(&self, param_json_bytes : &[u8]) -> Result<Vec<u8>, String> {
        unsafe {
            let generate_context = &self.generate_context_symbol;
            generate_context(param_json_bytes)
        }
    }
    pub fn deserialize_serialized_context(&self, context_bytes : &[u8]) -> Result<Context, String> {
        unsafe {
            let deserialize_context = &self.deserialize_context_symbol;
            deserialize_context(context_bytes)
        }
    }
}
