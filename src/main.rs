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

pub mod parsers;
pub mod expression;
pub mod bindings;

fn main() {
    pretty_env_logger::init();

    info!("TODO: do stuff");
}

