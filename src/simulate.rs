use fetish_lib::everything::*;
use crate::expression::*;
use ndarray::*;
use rand::*;

//All returned typed vectors are fully-expanded [not in the compressed space]
pub trait SimulatesExpressions {
    fn simulate_app_expression(&self, app_expr : AppExpression) -> Result<TypedVector, String>;
    fn simulate_func_expression(&self, func_expr : FuncExpression) -> Result<TypedVector, String>;
    fn simulate_expression(&self, expr : Expression) -> Result<TypedVector, String>;
}

fn expand_compressed_vector<'a>(zelf : &InterpreterAndEmbedderState<'a>, type_id : TypeId,
                                vec : Array1<f32>) -> TypedVector {
    let ctxt = zelf.get_context();
    if (ctxt.is_vector_type(type_id)) {
        TypedVector {
            type_id,
            vec
        }
    } else {
        let embedding_space = zelf.embedder_state.model_spaces.get(&type_id).unwrap();
        let elaborator_mean = embedding_space.elaborator.get_mean();
        let full_vec = elaborator_mean.dot(&vec);
        TypedVector {
            type_id,
            vec : full_vec
        }
    }
}

fn simulate_term_pointer<'a>(zelf : &InterpreterAndEmbedderState<'a>, term_ptr : TermPointer) -> TypedVector {
    let mut rng = rand::thread_rng();
    let model = zelf.embedder_state.get_embedding(term_ptr);
    let type_id = term_ptr.type_id;
    let vec = model.sample_as_vec(&mut rng);
    TypedVector {
        type_id,
        vec
    }
}

fn simulate_term_reference<'a>(zelf : &InterpreterAndEmbedderState<'a>, term_ref : TermReference) -> TypedVector {
    match (term_ref) {
        TermReference::FuncRef(func_ptr) => simulate_term_pointer(zelf, func_ptr),
        TermReference::VecRef(type_id, noisy_vec) => {
            let vec = from_noisy(noisy_vec.view());
            TypedVector {
                type_id,
                vec
            }
        }
    }
}

impl <'a> SimulatesExpressions for InterpreterAndEmbedderState<'a> {
    fn simulate_app_expression(&self, app_expr : AppExpression) -> Result<TypedVector, String> {
        let func_expr = *app_expr.func_expr;
        let arg_expr = *app_expr.arg_expr;

        let ctxt = self.get_context();

        let func_vec = self.simulate_func_expression(func_expr)?;
        let arg_vec = self.simulate_expression(arg_expr)?;

        let function_space_info = ctxt.get_function_space_info(func_vec.type_id);
        let arg_feat_info = ctxt.get_feature_space_info(arg_vec.type_id);

        let ret_type_id = ctxt.get_ret_type_id(func_vec.type_id);

        let func_mat = func_vec.vec.into_shape((function_space_info.get_output_dimensions(),
                                         function_space_info.get_feature_dimensions())).unwrap();

        let arg_feats = arg_feat_info.get_features_from_base(arg_vec.vec.view());

        let ret_compressed = func_mat.dot(&arg_feats);
        let ret = expand_compressed_vector(&self, ret_type_id, ret_compressed);
        Result::Ok(ret)
    }

    fn simulate_func_expression(&self, func_expr : FuncExpression) -> Result<TypedVector, String> {
        match (func_expr) {
            FuncExpression::Func(term_ptr) => Result::Ok(simulate_term_pointer(&self, term_ptr)),
            FuncExpression::App(app_expr) => {
                let formatted_app = format!("{}", &app_expr);
                let result_vec = self.simulate_app_expression(app_expr)?;
                if (self.get_context().is_vector_type(result_vec.type_id)) {
                    Result::Err(format!("Expected function, but obtained vector from simulating {}", 
                                            formatted_app))
                } else {
                    Result::Ok(result_vec)
                }
            }
        }
    }

    fn simulate_expression(&self, expr : Expression) -> Result<TypedVector, String> {
        match (expr) {
            Expression::Ref(term_ref) => Result::Ok(simulate_term_reference(&self, term_ref)),
            Expression::App(app_expression) => self.simulate_app_expression(app_expression)
        }
    }
}
