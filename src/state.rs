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
    pub ctxt_bytes : Vec<u8>,
    pub interpreter_and_embedder_state : SerializedInterpreterAndEmbedderState
}

impl ContextState {
    pub fn new(ctxt_bytes : Vec<u8>, ctxt : Context) -> ContextState {
        let deserialized_interpreter_and_embedder_state = InterpreterAndEmbedderState::new(&ctxt); 
        let interpreter_and_embedder_state = deserialized_interpreter_and_embedder_state.serialize();
        ContextState {
            ctxt,
            ctxt_bytes,
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

    pub fn perform_on_models<F, R>(&mut self, func : F) -> R
           where F : FnOnce(&mut InterpreterAndEmbedderState) -> R {

        let placeholder_interpreter_and_embedder_state = ContextState::make_empty_interpreter_and_embedder_state(); 

        let serialized_interpreter_and_embedder_state = mem::replace(&mut self.interpreter_and_embedder_state,
                                                                     placeholder_interpreter_and_embedder_state);

        let mut interpreter_and_embedder_state = serialized_interpreter_and_embedder_state.deserialize(&self.ctxt);

        let ret = func(&mut interpreter_and_embedder_state);

        let serialized_interpreter_and_embedder_state = interpreter_and_embedder_state.serialize();

        mem::replace(&mut self.interpreter_and_embedder_state, serialized_interpreter_and_embedder_state);

        ret
    }

    pub fn eval(&mut self, app_expr : Expression) -> Result<TermReference, String> {
        self.perform_on_models(|interpreter_and_embedder_state| 
                                interpreter_and_embedder_state.evaluate_expression(app_expr))
    }
    pub fn update_models(&mut self) {
        self.perform_on_models(|interpreter_and_embedder_state|
                               {
                                   interpreter_and_embedder_state.bayesian_update_step();
                                   interpreter_and_embedder_state.clear_newly_received();
                               });
    }
}

impl <'a> GlobalState<'a> {
    pub fn set_context(&mut self, ctxt_bytes : Vec<u8>, ctxt : Context) {
        let context_state = ContextState::new(ctxt_bytes, ctxt); 
        self.maybe_context_state = Option::Some(context_state);
    }
    pub fn unload_context(&mut self) {
        self.maybe_context_state = Option::None
    }
}
