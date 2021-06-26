use fetish_lib::everything::*;
use noisy_float::*;
use std::fmt;

#[derive(Clone)]
pub enum FuncExpression {
    Func(TermPointer),
    App(AppExpression)
}

#[derive(Clone)]
pub enum Expression {
    Ref(TermReference),
    App(AppExpression)
}

#[derive(Clone)]
pub struct AppExpression {
    func_expr : Box<FuncExpression>,
    arg_expr : Box<Expression>
}

impl AppExpression {
    pub fn maybe_from_expression(expr : Expression) -> Option<FuncExpression> {
        match (expr) {
            Expression::Ref(term_ref) => {
                match (term_ref) {
                    TermReference::FuncRef(func_ptr) => Option::Some(FuncExpression::Func(func_ptr)),
                    TermReference::VecRef(_, _) => Option::None
                }
            },
            Expression::App(app) => Option::Some(FuncExpression::App(app))
        }
    }
    pub fn new(func_expr : FuncExpression, arg_expr : Expression) -> AppExpression {
        AppExpression {
            func_expr : Box::new(func_expr),
            arg_expr : Box::new(arg_expr)
        }
    }
}

fn format_term_index(term_index : &TermIndex) -> String {
    match (term_index) {
        TermIndex::Primitive(ind) => format!("p{}", ind),
        TermIndex::NonPrimitive(ind) => format!("n{}", ind)
    }
}

fn format_term_ptr(term_ptr : &TermPointer) -> String {
    let term_ptr_str = format_term_index(&term_ptr.index);
    format!("#{}{}", term_ptr.type_id, term_ptr_str)
}

fn format_term_ref(term_ref : &TermReference) -> String {
    match (term_ref) {
        TermReference::FuncRef(func_ptr) => format_term_ptr(func_ptr),
        TermReference::VecRef(type_id, vec) => format!("#{}{}", type_id, vec)
    }
}

impl fmt::Display for FuncExpression {
    fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self) {
            FuncExpression::Func(term_ptr) => write!(f, "{}", format_term_ptr(term_ptr)),
            FuncExpression::App(app_expr) => app_expr.fmt(f)
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self) {
            Expression::Ref(term_ref) => write!(f, "{}", format_term_ref(term_ref)),
            Expression::App(app_expr) => app_expr.fmt(f)
        }
    }
}

impl fmt::Display for AppExpression {
    fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({} {})", self.func_expr, self.arg_expr)
    }
}

impl DisplayableWithState for Expression {
    fn display(&self, state : &InterpreterState) -> String {
        match (self) {
            Expression::Ref(term_ref) => term_ref.display(state),
            Expression::App(app_expr) => app_expr.display(state)
        }
    }
}

impl DisplayableWithState for AppExpression {
    fn display(&self, state : &InterpreterState) -> String {
        let func_str = self.func_expr.display(state);
        let arg_str = self.arg_expr.display(state);
        format!("({} {})", func_str, arg_str)
    }
}

impl DisplayableWithState for FuncExpression {
    fn display(&self, state : &InterpreterState) -> String {
        match (self) {
            FuncExpression::Func(term_ptr) => term_ptr.display(state),
            FuncExpression::App(app_expr) => app_expr.display(state)
        }
    }
}

pub fn build_application(mut expr_vec : Vec<Expression>) -> Result<AppExpression, String> {
    if (expr_vec.is_empty()) {
        return Result::Err("Empty expression".to_string());
    }
    if (expr_vec.len() < 2) {
        return Result::Err("Singleton expressions are disallowed".to_string());
    }
    let func_expr = expr_vec.remove(0);
    let maybe_func_expr = AppExpression::maybe_from_expression(func_expr);
    match (maybe_func_expr) {
        Option::None => Result::Err("Cannot apply vector as a function".to_string()),
        Option::Some(func_expr) => {
            let arg_expr = expr_vec.remove(0);
            let mut result = AppExpression::new(func_expr, arg_expr);
            for arg_expr in expr_vec.drain(..) {
                let func_expr = FuncExpression::App(result);
                result = AppExpression::new(func_expr, arg_expr);
            }
            Result::Ok(result)
        }
    }
}

pub trait EvaluatesExpressions {
    fn evaluate_app_expression(&mut self, app_expr : AppExpression) -> Result<TermReference, String>;
    fn evaluate_func_expression(&mut self, func_expr : FuncExpression) -> Result<TermPointer, String>;
    fn evaluate_expression(&mut self, expr : Expression) -> Result<TermReference, String>;
}

impl <'a> EvaluatesExpressions for InterpreterAndEmbedderState<'a> {
    fn evaluate_app_expression(&mut self, app_expr : AppExpression) -> Result<TermReference, String> {
        let func_expr = *app_expr.func_expr;
        let arg_expr = *app_expr.arg_expr;
        
        let func_ptr = self.evaluate_func_expression(func_expr)?;
        let arg_ref = self.evaluate_expression(arg_expr)?;
        let term_app = TermApplication {
            func_ptr,
            arg_ref
        };
        
        let result_ref = self.evaluate(&term_app);
        Result::Ok(result_ref)
    }
    fn evaluate_func_expression(&mut self, func_expr : FuncExpression) -> Result<TermPointer, String> {
        match (func_expr) {
            FuncExpression::Func(term_ptr) => Result::Ok(term_ptr),
            FuncExpression::App(app_expr) => {
                let formatted_app = format!("{}", &app_expr);
                let result_ref = self.evaluate_app_expression(app_expr)?;
                match (result_ref) {
                    TermReference::VecRef(_, _) => {
                        Result::Err(format!("Expected function, but obtained vector from evaluating {}", 
                                            formatted_app))
                    },
                    TermReference::FuncRef(func_ptr) => Result::Ok(func_ptr)
                }
            }
        }
    }
    fn evaluate_expression(&mut self, expr : Expression) -> Result<TermReference, String> {
        match (expr) {
            Expression::Ref(term_ref) => Result::Ok(term_ref),
            Expression::App(app_expression) => self.evaluate_app_expression(app_expression)
        }
    }
}
