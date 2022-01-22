use std::{rc::Rc, sync::RwLock};

use crate::{
    exceptions::RuntimeException,
    interpreter::Interpreter,
    object::{CallableLoxObject, LoxNil, LoxObject, LoxObjectBase, PrimitiveLoxObject},
    stmt::FunStmt,
};

#[derive(Clone)]
pub struct LoxNativeCallable {
    pub arity: usize,
    pub call_fun: fn(&Interpreter, Vec<LoxObject>) -> Result<LoxObject, RuntimeException>,
}

impl LoxNativeCallable {
    pub fn new(
        arity: usize,
        call_fun: fn(&Interpreter, Vec<LoxObject>) -> Result<LoxObject, RuntimeException>,
    ) -> LoxObject {
        LoxObject(Rc::new(RwLock::new(LoxNativeCallable { arity, call_fun })))
    }
}

impl std::fmt::Debug for LoxNativeCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoxNativeCallable")
            .field("arity", &self.arity)
            .finish()
    }
}

impl LoxObjectBase for LoxNativeCallable {}
impl PrimitiveLoxObject for LoxNativeCallable {
    fn instance_name(&self) -> &'static str {
        "NativeCallable"
    }
}
impl CallableLoxObject for LoxNativeCallable {
    fn arity_self(&self) -> usize {
        self.arity
    }

    fn call_self(
        &self,
        interpreter: &Interpreter,
        arguments: Vec<LoxObject>,
    ) -> Result<LoxObject, RuntimeException> {
        (self.call_fun)(interpreter, arguments)
    }
}

#[derive(Debug, Clone)]
pub struct LoxFunction {
    pub declaration: FunStmt,
}

impl LoxFunction {
    pub fn new(declaration: &FunStmt) -> LoxObject {
        LoxObject(Rc::new(RwLock::new(LoxFunction {
            declaration: declaration.clone(),
        })))
    }
}

impl LoxObjectBase for LoxFunction {}
impl PrimitiveLoxObject for LoxFunction {
    fn instance_name(&self) -> &'static str {
        "Function"
    }
}

impl CallableLoxObject for LoxFunction {
    fn arity_self(&self) -> usize {
        self.declaration.params.len()
    }

    fn call_self(
        &self,
        interpreter: &Interpreter,
        arguments: Vec<LoxObject>,
    ) -> Result<LoxObject, RuntimeException> {
        interpreter.environment.enter_function_scope();
        interpreter.environment.enter_new_scope();
        // This would typically be able to panic, but because we're checking the
        // arity and the arguments beforehand, we're good.

        for i in 0..self.declaration.params.len() {
            interpreter.environment.define(
                &self.declaration.params.get(i).unwrap().lexeme,
                arguments.get(i).unwrap().clone(),
            );
        }

        // Execute our function in the correct scope.
        let execution_result = interpreter.execute_block(&self.declaration.body);
        // Return to the normal environment's scope.
        interpreter.environment.exit_function_scope();

        match execution_result {
            Err(RuntimeException::RuntimeError(err)) => {
                // There was a runtime error
                return Err(RuntimeException::RuntimeError(err));
            }
            Err(RuntimeException::ReturnException(return_exception)) => {
                // Early return value emitted
                return Ok(return_exception.value);
            }
            // No return value was emitted
            _ => Ok(LoxNil::new()),
        }
    }
}
