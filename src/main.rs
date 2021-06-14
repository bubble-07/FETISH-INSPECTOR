#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(unused_parens)]

extern crate ndarray;
extern crate libloading;
extern crate ndarray_linalg;
extern crate pretty_env_logger;
extern crate fetish_lib;
#[macro_use] extern crate log;

use ndarray::*;
use rand::prelude::*;
use fetish_lib::everything::*;
use crate::expression::*;
use crate::parsers::*;
use crate::bindings::*;
use std::env;
use libloading::{Library, Symbol};
use crate::loading::*;

use rustyline::error::ReadlineError;
use rustyline::Editor;

pub mod parsers;
pub mod expression;
pub mod bindings;
pub mod loading;

fn main() {
    let args : Vec<String> = env::args().collect();
    if (args.len() < 2) {
        println!("Usage: first and only argument is path to context generator dylib");
        return;
    }

    let context_generator_path = &args[1];

    let mut bindings = Bindings::new();

    let maybe_lib = unsafe {
        Library::new(context_generator_path)
    };
    
    match (maybe_lib) {
        Result::Err(err) => {
            println!("Error occurred when loading context generation library: {}", err);
            return;
        },
        Result::Ok(context_generation_lib) => {
            let maybe_lib_handle = ContextDefinitionLibraryHandle::new(&context_generation_lib);
            match (maybe_lib_handle) {
                Result::Err(err) => {
                    println!("Error occurred while locating context generation library symbols: {}", err);
                    return;
                },
                Result::Ok(lib_handle) => {
                    let mut rl = Editor::<()>::new();
                    loop {
                        let readline = rl.readline(">> ");
                        match readline {
                            Ok(line) => {
                                handle_command(line.as_str(), &lib_handle, &mut bindings);
                                rl.add_history_entry(line.as_str());
                            },
                            Err(ReadlineError::Interrupted) => {
                                println!("CTRL-C");
                                break
                            },
                            Err(ReadlineError::Eof) => {
                                println!("CTRL-D");
                                break
                            },
                            Err(err) => {
                                println!("Error: {:?}", err);
                                break
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn handle_command<'a>(line : &str, lib_handle : &ContextDefinitionLibraryHandle<'a>,
                          bindings : &mut Bindings) {
    let parse_result = parse_s_expression(line, &bindings);
    match (parse_result) {
        Result::Ok((expr, _)) => {
            println!("{}", expr);
        },
        Result::Err(err) => {
            println!("Error: {}", err);
        }
    }
}
