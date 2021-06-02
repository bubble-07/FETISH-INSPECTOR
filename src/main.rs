#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(unused_parens)]

extern crate ndarray;
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

use rustyline::error::ReadlineError;
use rustyline::Editor;

pub mod parsers;
pub mod expression;
pub mod bindings;

fn main() {
    let mut bindings = Bindings::new();

    let mut rl = Editor::<()>::new();

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let parse_result = parse_s_expression(line.as_str(), &bindings);
                match (parse_result) {
                    Result::Ok((expr, _)) => {
                        println!("{}", expr);
                    },
                    Result::Err(err) => {
                        println!("Error: {}", err);
                    }
                }
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

