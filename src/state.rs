use crate::bindings::*;
use crate::loading::*;
use fetish_lib::everything::*;
use crate::expression::*;
use std::collections::HashMap;
use std::mem;

pub struct GlobalState<'a> {
    pub bindings : Bindings,
    pub lib_handle : ContextDefinitionLibraryHandle<'a>,
    pub maybe_context_state : Option<ContextState>
}

pub struct ContextState {
    pub ctxt : Context,
    pub interpreter_and_embedder_state : SerializedInterpreterAndEmbedderState
}

impl ContextState {
    pub fn new(ctxt : Context) -> ContextState {
        let deserialized_interpreter_and_embedder_state = InterpreterAndEmbedderState::new(&ctxt); 
        let interpreter_and_embedder_state = deserialized_interpreter_and_embedder_state.serialize();
        ContextState {
            ctxt,
            interpreter_and_embedder_state
        }
    }

    fn make_empty_interpreter_and_embedder_state() -> SerializedInterpreterAndEmbedderState {
        let newly_evaluated_terms = NewlyEvaluatedTerms::new();
        let interpreter_state = SerializedInterpreterState {
            application_tables : HashMap::new(),
            type_spaces : HashMap::new()
        };
        let embedder_state = SerializedEmbedderState {
            model_spaces : HashMap::new()
        };
        SerializedInterpreterAndEmbedderState {
            interpreter_state,
            embedder_state,
            newly_evaluated_terms
        }
    }

    pub fn eval(&mut self, app_expr : AppExpression) -> Result<TermReference, String> {
        let placeholder_interpreter_and_embedder_state = ContextState::make_empty_interpreter_and_embedder_state(); 

        let serialized_interpreter_and_embedder_state = mem::replace(&mut self.interpreter_and_embedder_state,
                                                                     placeholder_interpreter_and_embedder_state);

        let mut interpreter_and_embedder_state = serialized_interpreter_and_embedder_state.deserialize(&self.ctxt);

        let ret = interpreter_and_embedder_state.evaluate_app_expression(app_expr);

        let serialized_interpreter_and_embedder_state = interpreter_and_embedder_state.serialize();

        mem::replace(&mut self.interpreter_and_embedder_state, serialized_interpreter_and_embedder_state);

        ret
    }
}

impl <'a> GlobalState<'a> {
    pub fn set_context(&mut self, ctxt : Context) {
        let context_state = ContextState::new(ctxt); 
        self.maybe_context_state = Option::Some(context_state);
    }
    pub fn unload_context(&mut self) {
        self.maybe_context_state = Option::None
    }
}
