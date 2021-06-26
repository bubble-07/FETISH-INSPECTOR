use fetish_lib::everything::*;
use std::collections::HashMap;
use crate::expression::*;

pub struct Bindings {
    bindings : HashMap<String, TermReference>
}

impl Bindings {
    pub fn write(&mut self, identifier : String, term_ref : TermReference) {
        self.bindings.insert(identifier, term_ref);
    }
    pub fn lookup<'a>(&self, identifier : &'a str) -> Result<TermReference, String> {
        let identifier_as_string = identifier.to_string();
        let maybe_result = self.bindings.get(&identifier_as_string);
        match (maybe_result) {
            Option::None => Result::Err(format!("No identifier named {} in scope", identifier)),
            Option::Some(result) => Result::Ok(result.clone())
        }
    }
    
    pub fn new() -> Bindings {
        Bindings {
            bindings : HashMap::new()
        }
    }
}
