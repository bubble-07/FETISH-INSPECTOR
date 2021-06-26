use std::fs;
use crate::bindings::*;
use crate::state::*;
use crate::parsers::*;
use fetish_lib::everything::*;

pub enum Command {
    Contextual(ContextualCommand),
    Parse(String),
    GenerateContextFromPath(String),
    LoadContextFromPath(String),
    UnloadContext
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
            Command::UnloadContext => glob_state.unload_context()
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

pub fn handle_save_models(path : String, context_state : &ContextState) {
    //TODO: implement
    panic!();
}

pub fn handle_load_models(path : String, context_state : &mut ContextState, bindings : &mut Bindings) {
    //TODO: implement
    panic!();
}

pub fn handle_save_context(path : String, context_state : &ContextState) {
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
    let parse_result = parse_s_expression(&expr_text, &*bindings);
    match (parse_result) {
        Result::Err(err) => {
            println!("Let: Expression Parsing Error: {}", err);
        },
        Result::Ok((app_expr, _)) => {
            let maybe_result_ref = context_state.eval(app_expr);            
            match (maybe_result_ref) {
                Result::Err(err) => {
                    println!("Let: Expression Evaluation Error: {}", err);
                },
                Result::Ok(result_ref) => {
                    bindings.write(var_text, result_ref);
                }
            }
        }
    }
}

pub fn handle_load_context(path : String, glob_state : &mut GlobalState) {
    let maybe_path_contents = fs::read_to_string(path);
    match (maybe_path_contents) {
        Result::Err(err) => {
            println!("Load Context: IO Error: {}", err);
        },
        Result::Ok(path_contents) => {
            let maybe_context = glob_state.lib_handle.deserialize_context_json(&path_contents);
            match (maybe_context) {
                Result::Err(err) => {
                    println!("Load Context: JSON Error: {}", err);
                },
                Result::Ok(context) => {
                    glob_state.set_context(context);
                }
            }
        }
    }
}

pub fn handle_generate_context(path : String, glob_state : &mut GlobalState) {
    let maybe_path_contents = fs::read_to_string(path);    
    match (maybe_path_contents) {
        Result::Err(err) => {
            println!("Generate Context: IO Error: {}", err);
        },
        Result::Ok(path_contents) => {
            let maybe_context_json = glob_state.lib_handle.generate_context_json(&path_contents);
            match (maybe_context_json) {
                Result::Err(err) => {
                    println!("Generate Context (Generate): JSON Error: {}", err);
                },
                Result::Ok(context_json) => {
                    let maybe_context = glob_state.lib_handle.deserialize_context_json(&context_json);
                    match (maybe_context) {
                        Result::Err(err) => {
                            println!("Generate Context (Deserialize): JSON Error: {}", err);
                        },
                        Result::Ok(context) => {
                            glob_state.set_context(context);
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
