use fetish_lib::everything::*;
use ndarray::*;
use noisy_float::prelude::*;
use crate::expression::*;
use crate::bindings::*;
use crate::commands::*;

pub fn parse_command_line(text : &str) -> Result<Command, String> {
    let trimmed_text = text.trim();
    let maybe_split_text = trimmed_text.split_once(' ');
    match (maybe_split_text) {
        Option::Some((command_text, rest_text)) => {
            let trimmed_rest = rest_text.trim();
            parse_argumented_command(command_text, trimmed_rest)
        },
        Option::None => {
            parse_primitive_command(trimmed_text) 
        }
    }
}

pub fn parse_primitive_command(command_text : &str) -> Result<Command, String> {
    match (command_text) {
        "unload_context" => Result::Ok(Command::UnloadContext),
        "list_types" => Result::Ok(Command::Contextual(ContextualCommand::ListTypes)),
        "help" => Result::Ok(Command::Help),
        _ => Result::Err(format!("{} is not a recognized command (without arguments)", command_text))
    }
}

pub fn parse_argumented_command(command_text : &str, trimmed_rest : &str) -> Result<Command, String> {
    let rest = trimmed_rest.to_owned();
    match (command_text) {
        "parse" => Result::Ok(Command::Parse(rest)),
        "generate_context" => Result::Ok(Command::GenerateContextFromPath(rest)),
        "load_context" => Result::Ok(Command::LoadContextFromPath(rest)),
        "let" => parse_let(trimmed_rest),
        "evaluate" | "eval" => Result::Ok(Command::Contextual(ContextualCommand::Evaluate(rest))),
        "simulate" | "sim" => Result::Ok(Command::Contextual(ContextualCommand::Simulate(rest))),
        "list_primitive_terms" | "list_prim_terms" => 
                        Result::Ok(Command::Contextual(ContextualCommand::ListPrimitiveTerms(rest))),
        "save_context" => Result::Ok(Command::Contextual(ContextualCommand::SaveContextToPath(rest))),
        "load_models" => Result::Ok(Command::Contextual(ContextualCommand::LoadModelsFromPath(rest))),
        "save_models" => Result::Ok(Command::Contextual(ContextualCommand::SaveModelsToPath(rest))),
        _ => Result::Err(format!("{} is not a recognized command (with arguments)", command_text))
    }
}

pub fn parse_let(let_body_text : &str) -> Result<Command, String> {
    let maybe_split = let_body_text.split_once('=');
    match (maybe_split) {
        Option::Some((untrimmed_var_text, untrimmed_expr_text)) => {
            let var_text = untrimmed_var_text.trim();
            let expr_text = untrimmed_expr_text.trim();
            Result::Ok(Command::Contextual(ContextualCommand::Let(var_text.to_owned(), expr_text.to_owned())))
        },
        Option::None => 
            Result::Err(format!("Let body {} does not have the format [var] = [expr]", let_body_text))
    }
}

///(_* [func_atom] [arg_atom_1] ... [arg_atom_n] _*)
pub fn parse_s_expression<'a>(text : &'a str, bindings : &Bindings) -> Result<(AppExpression, &'a str), String> {
    let maybe_without_left_paren = text.strip_prefix('(');
    match (maybe_without_left_paren) {
        Option::None => Result::Err(format!("Missing left paren for sub-expression: {}", text)),
        Option::Some(without_left_paren) => {
            let interior_to_end = without_left_paren.trim_start();

            let mut atom_exprs = Vec::new();
            let mut current_text = interior_to_end;
            while (!current_text.starts_with(')')) {
                if (current_text.is_empty()) {
                    return Result::Err(format!("Ran out of input while parsing s-expression: {}", text));
                }
                let (atom_expr, modified_text) = parse_atom(current_text, bindings)?;
                current_text = modified_text.trim_start();
                atom_exprs.push(atom_expr);
            }

            current_text = current_text.strip_prefix(')').unwrap();

            let result_expr = build_application(atom_exprs)?;
            Result::Ok((result_expr, current_text))
        }
    }
}

pub fn parse_reference<'a>(text : &'a str) -> Result<(TermReference, &'a str), String> {
    let maybe_without_pound_sign = text.strip_prefix('#'); 
    match (maybe_without_pound_sign) {
        Option::None => Result::Err(format!("Missing pound sign for reference: {}", text)),
        Option::Some(without_pound_sign) => {
            let maybe_first_nonnumeral_index = without_pound_sign.find(|c : char| !c.is_digit(10));
            match (maybe_first_nonnumeral_index) {
                Option::None => Result::Err(format!("Ran out of input while parsing reference: {}", text)),
                Option::Some(first_nonnumeral_index) => {
                    let (prefix_numeral_str, suffix) = without_pound_sign.split_at(first_nonnumeral_index);
                    let type_number : usize = str::parse(prefix_numeral_str).unwrap();
                    if (suffix.starts_with('[')) {
                        let (vec, remaining_text) = parse_vector(suffix)?;
                        Result::Ok((TermReference::VecRef(type_number, vec), remaining_text))
                    } else {
                        let (term_index, remaining_text) = parse_term_index(suffix)?;
                        let term_ptr = TermPointer {
                            type_id : type_number,
                            index : term_index
                        };
                        Result::Ok((TermReference::FuncRef(term_ptr), remaining_text))
                    } 
                }
            }
        }
    }    
}

pub fn parse_term_index<'a>(text : &'a str) -> Result<(TermIndex, &'a str), String> {
    let is_primitive = text.starts_with('p');
    if (!text.starts_with('p') && !text.starts_with('n')) {
        return Result::Err(format!("Cannot parse term index {}", text));
    }
    let starting_char = if (is_primitive) {'p'} else {'n'};
    let starting_with_numeral = text.strip_prefix(starting_char).unwrap();
    let maybe_first_nonnumeral_index = starting_with_numeral.find(|c : char| !c.is_digit(10));
    match (maybe_first_nonnumeral_index) {
        Option::None => Result::Err(format!("Ran out of input while parsing term index: {}", text)),
        Option::Some(first_nonnumeral_index) => {
            let (prefix_numeral_str, suffix) = starting_with_numeral.split_at(first_nonnumeral_index);
            let term_number : usize = str::parse(prefix_numeral_str).unwrap();
            let term_index = if (is_primitive) {
                                TermIndex::Primitive(term_number)
                             } else {
                                TermIndex::NonPrimitive(term_number)
                             };
            Result::Ok((term_index, suffix))
        }
    }
}

pub fn parse_vector<'a>(text : &'a str) -> Result<(Array1<R32>, &'a str), String> {
    let maybe_without_left_bracket = text.strip_prefix('[');
    match (maybe_without_left_bracket) {
        Option::None => Result::Err(format!("Missing left bracket for vector: {}", text)),
        Option::Some(without_left_bracket) => {
            let maybe_right_bracket_index = without_left_bracket.find(']'); 
            match (maybe_right_bracket_index) {
                Option::None => Result::Err(format!("Missing right bracket for vector: {}", text)),
                Option::Some(right_bracket_index) => {
                    let (vector_content_str, right_bracket_remainder) = 
                                             without_left_bracket.split_at(right_bracket_index);
                    let remainder = right_bracket_remainder.strip_prefix(']').unwrap();

                    let mut elems = Vec::new();
                    for padded_elem_str in vector_content_str.split(',') {
                        let elem_str = padded_elem_str.trim(); 
                        let maybe_elem_as_float = str::parse(elem_str);
                        match (maybe_elem_as_float) {
                            Result::Err(_) => {
                                return Result::Err(format!("Malformed float: {}", elem_str));
                            },
                            Result::Ok(elem_as_float) => {
                                let elem = r32(elem_as_float);
                                elems.push(elem);
                            }
                        }
                    }
                    let vec = Array1::from(elems);
                    Result::Ok((vec, remainder))
                }
            }
        }
    }
}

pub fn parse_identifier<'a>(text : &'a str, bindings : &Bindings) -> Result<(TermReference, &'a str), String> {
    let maybe_identifier_end_index = text.find(|c : char| c.is_whitespace() || c == ')');
    match (maybe_identifier_end_index) {
        Option::None => Result::Err(format!("Ran out of input while parsing identifier: {}", text)),
        Option::Some(identifier_end_index) => {
            let (identifier_content_str, suffix) = text.split_at(identifier_end_index);
            let term_ref = bindings.lookup(identifier_content_str)?;
            Result::Ok((term_ref, suffix))
        } 
    }
}

pub fn parse_atom<'a>(text : &'a str, bindings : &Bindings) -> Result<(Expression, &'a str), String> {
    if (text.is_empty()) {
        return Result::Err("Cannot parse atom: empty text".to_string());
    }
    if (text.starts_with('(')) {
        let (app_expr, updated_text) = parse_s_expression(text, bindings)?;
        let expr = Expression::App(app_expr);
        return Result::Ok((expr, updated_text));
    } else if (text.starts_with('#')) {
        let (ref_expr, updated_text) = parse_reference(text)?;
        let expr = Expression::Ref(ref_expr);
        return Result::Ok((expr, updated_text));
    } else {
        //Must be an identifier
        let (ref_expr, updated_text) = parse_identifier(text, bindings)?;
        let expr = Expression::Ref(ref_expr); 
        return Result::Ok((expr, updated_text));
    }
}
