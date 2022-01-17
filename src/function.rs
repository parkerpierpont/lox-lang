use std::{rc::Rc, sync::RwLock};

use crate::{
    interpreter::Interpreter,
    object::{CallableLoxObject, LoxNil, LoxObject, LoxObjectBase, PrimitiveLoxObject},
    runtime_error::RuntimeError,
    stmt::FunStmt,
};

pub struct LoxNativeCallable {
    pub arity: usize,
    pub call_fun: fn(&Interpreter, Vec<LoxObject>) -> Result<LoxObject, RuntimeError>,
}

impl LoxNativeCallable {
    pub fn new(
        arity: usize,
        call_fun: fn(&Interpreter, Vec<LoxObject>) -> Result<LoxObject, RuntimeError>,
    ) -> LoxObject {
        LoxObject(Rc::new(RwLock::new(LoxNativeCallable { arity, call_fun })))
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
    ) -> Result<LoxObject, RuntimeError> {
        (self.call_fun)(interpreter, arguments)
    }
}

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
    ) -> Result<LoxObject, RuntimeError> {
        let env = interpreter.environment.new_from_globals();
        env.enter_new_scope();
        // This would typically be able to panic, but because we're checking the
        // arity and the arguments beforehand, we're good.
        for i in 0..self.declaration.params.len() {
            env.define(
                &self.declaration.params.get(i).unwrap().lexeme,
                arguments.get(i).unwrap().clone(),
            );
        }

        let env = env.into_env_base();
        let old_env = interpreter.environment.replace_scope(env);

        if let Err(runtime_error) = interpreter.execute_block(&self.declaration.body) {
            return Err(runtime_error);
        }

        interpreter.environment.replace_scope(old_env);

        return Ok(LoxNil::new());
    }
}
