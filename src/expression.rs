use fetish_lib::everything::*;
use noisy_float::*;

pub enum FuncExpression {
    Func(TermPointer),
    App(AppExpression)
}

pub enum Expression {
    Ref(TermReference),
    App(AppExpression)
}

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

