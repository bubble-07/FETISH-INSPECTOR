use std::fs;
use crate::bindings::*;
use crate::state::*;
use crate::parsers::*;
use crate::expression::*;
use fetish_lib::everything::*;

pub enum Command {
    Contextual(ContextualCommand),
    Parse(String),
    GenerateContextFromPath(String),
    LoadContextFromPath(String),
    UnloadContext,
    Help
}

pub enum ContextualCommand {
    Let(String, String),
    Evaluate(String),
    Simulate(String),
    ListTypes,
    ListPrimitiveTerms(String),
    SaveContextToPath(String),
    LoadModelsFromPath(String),
    SaveModelsToPath(String)
}

impl Command {
    pub fn handle_command<'a>(self, glob_state : &mut GlobalState<'a>) {
        match (self) {
            Command::Contextual(context_command) => context_command.handle_command(glob_state),
            Command::Parse(text) => handle_parse(text, &glob_state.bindings),
            Command::GenerateContextFromPath(path) => handle_generate_context(path, glob_state),
            Command::LoadContextFromPath(path) => handle_load_context(path, glob_state),
            Command::UnloadContext => glob_state.unload_context(),
            Command::Help => handle_help()
        }
    }
}

impl ContextualCommand {
    pub fn handle_command<'a>(self, glob_state : &mut GlobalState<'a>) {
        let bindings = &mut glob_state.bindings;
        match (&mut glob_state.maybe_context_state) {
            Option::None => {
                println!("This command may not be executed without a currently-loaded context");
            },
            Option::Some(context_state) => {
                match (self) {
                    ContextualCommand::Let(var_text, expr_text)
                                     => handle_let(var_text, expr_text, context_state, bindings),
                    ContextualCommand::Evaluate(expr_text)
                                     => handle_evaluate(expr_text, context_state, bindings),
                    ContextualCommand::Simulate(expr_text)
                                     => handle_simulate(expr_text, context_state, &*bindings),
                    ContextualCommand::ListTypes
                                     => handle_list_types(&*context_state),
                    ContextualCommand::ListPrimitiveTerms(type_text)
                                     => handle_list_primitive_terms(type_text, &*context_state),
                    ContextualCommand::SaveContextToPath(path)
                                     => handle_save_context(path, &*context_state),
                    ContextualCommand::LoadModelsFromPath(path)
                                     => handle_load_models(path, context_state, bindings),
                    ContextualCommand::SaveModelsToPath(path)
                                     => handle_save_models(path, &*context_state)
                }
            }
        }
    }
}

pub fn handle_help() {
    println!("generate_context [path]: Generates a Context from the path to json-ized Params to generate it from");
    println!("load_context [path]: Loads a json-ized Context from the given path");
    println!("unload_context: Unloads the current Context");
    println!("list_types: Lists all types matching type numbers to their definitions");
    println!("parse [expr]: Parses the given s-expression, and renders what it parsed as");
    println!("let [var] = [expr]: Evaluates the expression, and binds it to the given variable");
    println!("eval [expr] | evaluate [expr]: Evaluates the expression, and prints the result");
    println!("simulate [expr] | sim [expr]: Simulates the given expression [via a drawn sample], and prints the result");
    println!("list_primitive_terms [type_num] | list_prim_terms [type_num]: Lists the primitive terms of the type with the given number");
    println!("save_context [path]: Saves the current Context, json-ized, to the given path");
    println!("load_models [path]: Loads the jsonized interpreter+embedder state from the given path");
    println!("save_models [path]: Saves the interpreter+embedder state as json to the given path");
    println!("help: Prints this help screen");
}

pub fn handle_save_models(path : String, context_state : &ContextState) {
    //TODO: implement
    panic!();
}

pub fn handle_load_models(path : String, context_state : &mut ContextState, bindings : &mut Bindings) {
    //TODO: implement
    panic!();
}


pub fn handle_list_primitive_terms(type_text : String, context_state : &ContextState) {
    let maybe_type_number = type_text.trim().parse::<usize>();
    match (maybe_type_number) {
        Result::Err(err) => {
            println!("List Primitive Terms: Unable to parse type number from {}", type_text);
            println!("Due to error: {}", err);
        },
        Result::Ok(type_id) => {
            let primitive_directory = &context_state.ctxt.primitive_directory;
            let primitive_type_space = primitive_directory.primitive_type_spaces.get(&type_id).unwrap();
            for i in 0..primitive_type_space.terms.len() {
                let term_name = primitive_type_space.terms[i].get_name();
                println!("p{}: {}", i, term_name);
            }
        }
    }
    let ctxt = &context_state.ctxt;
}

pub fn handle_list_types(context_state : &ContextState) {
    let ctxt = &context_state.ctxt;
    for type_id in 0..ctxt.get_total_num_types() {
        let kind = ctxt.get_type(type_id);
        let type_text = kind.display(ctxt);
        println!("#{}: {}", type_id, type_text);
    }
}

pub fn handle_simulate(expr_text : String, context_state : &mut ContextState, bindings : &Bindings) {
    //TODO: Implement me!
    panic!();
}

pub fn handle_evaluate(expr_text : String, context_state : &mut ContextState, bindings : &mut Bindings) {
    handle_let("ans".to_owned(), expr_text, context_state, bindings);
}

pub fn handle_let(var_text : String, expr_text : String, 
                  context_state : &mut ContextState, bindings : &mut Bindings) {
    let parse_result = parse_atom(&expr_text, &*bindings);
    match (parse_result) {
        Result::Err(err) => {
            println!("Let: Expression Parsing Error: {}", err);
        },
        Result::Ok((expr, _)) => {
            let maybe_result_ref = context_state.eval(expr);            
            match (maybe_result_ref) {
                Result::Err(err) => {
                    println!("Let: Expression Evaluation Error: {}", err);
                },
                Result::Ok(result_ref) => {
                    let result_string = format_term_ref(&result_ref);
                    bindings.write(var_text, result_ref);
                    println!("{}", &result_string);
                }
            }
        }
    }
}

pub fn handle_save_context(path : String, context_state : &ContextState) {
    let maybe_write_result = write_to_path(&path, &context_state.ctxt_bytes); 
    match (maybe_write_result) {
        Result::Ok(_) => {
            println!("Successfully wrote out context JSON");
        },
        Result::Err(err) => {
            println!("Failed to write out context JSON: {}", err);
        }
    }
}

pub fn write_to_path(path : &str, contents : &[u8]) -> Result<(), String> {
    let maybe_canonical_path = shellexpand::full(path);
    match (maybe_canonical_path) {
        Result::Ok(canonical_path) => {
            let maybe_write_result = fs::write(&*canonical_path, contents);
            match (maybe_write_result) {
                Result::Ok(_) => Result::Ok(()),
                Result::Err(err) => Result::Err(format!("Writing Error: {}", err))
            }
        },
        Result::Err(err) => Result::Err(format!("Path Resolution Error: {}", err))
    }
}

pub fn read_from_path(path : String) -> Result<Vec<u8>, String> {
    let maybe_canonical_path = shellexpand::full(&path);
    match (maybe_canonical_path) {
        Result::Ok(canonical_path) => {
            let maybe_path_contents = fs::read(&*canonical_path);
            match (maybe_path_contents) {
                Result::Ok(path_contents) => Result::Ok(path_contents),
                Result::Err(err) => Result::Err(format!("Read Error: {}", err))
            }
        },
        Result::Err(err) => Result::Err(format!("Path Resolution Error: {}", err))
    }
}

pub fn handle_load_context(path : String, glob_state : &mut GlobalState) {
    let maybe_path_contents = read_from_path(path);
    match (maybe_path_contents) {
        Result::Err(err) => {
            println!("Load Context: IO Error: {}", err);
        },
        Result::Ok(path_contents) => {
            let maybe_context = glob_state.lib_handle.deserialize_serialized_context(&path_contents);
            match (maybe_context) {
                Result::Err(err) => {
                    println!("Load Context: JSON Error: {}", err);
                },
                Result::Ok(context) => {
                    glob_state.set_context(path_contents, context);
                }
            }
        }
    }
}

pub fn handle_generate_context(path : String, glob_state : &mut GlobalState) {
    let maybe_path_contents = read_from_path(path);
    match (maybe_path_contents) {
        Result::Err(err) => {
            println!("Generate Context: IO Error: {}", err);
        },
        Result::Ok(path_contents) => {
            let maybe_context_json = glob_state.lib_handle.generate_serialized_context(&path_contents);
            match (maybe_context_json) {
                Result::Err(err) => {
                    println!("Generate Context (Generate): JSON Error: {}", err);
                },
                Result::Ok(context_json) => {
                    let maybe_context = glob_state.lib_handle.deserialize_serialized_context(&context_json);
                    match (maybe_context) {
                        Result::Err(err) => {
                            println!("Generate Context (Deserialize): JSON Error: {}", err);
                        },
                        Result::Ok(context) => {
                            glob_state.set_context(context_json, context);
                        }
                    }
                }
            }
        }
    }
}

pub fn handle_parse(line : String, bindings : &Bindings) {
    let parse_result = parse_s_expression(line.as_str(), &bindings);
    match (parse_result) {
        Result::Ok((expr, _)) => {
            println!("{}", expr);
        },
        Result::Err(err) => {
            println!("Parsing error: {}", err);
        }
    }
}
