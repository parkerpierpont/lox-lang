use crate::{object::LoxObject, runtime_error::RuntimeError, token::Token};
use downcast::{downcast, Any};
use std::{collections::HashMap, rc::Rc, sync::RwLock};

pub trait EnvironmentTrait: Any {
    fn define(&mut self, name: &String, value: LoxObject);
    fn get(&self, name: &Token) -> Result<LoxObject, RuntimeError>;
    fn assign(&mut self, name: &Token, value: LoxObject) -> Result<(), RuntimeError>;
    fn is_global(&self) -> bool;
    fn take_enclosing_scope(self) -> Option<Box<EnvironmentBase>>;
}

pub struct EnvironmentBase {
    values: HashMap<String, LoxObject>,
    enclosing: Option<Environment>,
}

downcast!(dyn EnvironmentTrait);
pub type Environment = Box<dyn EnvironmentTrait>;

impl EnvironmentBase {
    /// Create a new environment.
    pub fn new_global() -> EnvironmentBase {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    /// Create a new environment.
    pub fn new_scoped(enclosing: Environment) -> EnvironmentBase {
        Self {
            values: HashMap::new(),
            enclosing: Some(enclosing),
        }
    }
}

impl EnvironmentTrait for EnvironmentBase {
    /// Define a variable.
    fn define(&mut self, name: &String, value: LoxObject) {
        // Because we don't check to see if the name exists yet, we're able to
        // redefine variables in a single environment.
        self.values.insert(name.clone(), value);
    }

    /// Get a variable's value. This will return a runtime error if the variable
    /// hasn't been set yet.
    ///
    /// We have to do this at runtime to support lazy references to variables in
    /// functions. We could statically check all of this (I believe) – but it's
    /// too involved for this tutorial.
    fn get(&self, name: &Token) -> Result<LoxObject, RuntimeError> {
        match self.values.get(&name.lexeme) {
            Some(value) => Ok(value.clone()),
            None => {
                if let Some(enclosing) = self.enclosing.as_ref() {
                    return enclosing.get(name);
                }

                Err(RuntimeError::new(
                    name.clone(),
                    format!("Undefined variable '{}'.", name.lexeme),
                ))
            }
        }
    }

    /// Similar to 'get', but this doesn't let you create a new variable. If a
    /// new variable creation is attempted, this will throw a 'RuntimeError'.
    fn assign(&mut self, name: &Token, value: LoxObject) -> Result<(), RuntimeError> {
        // If the key exists, replace it with new value.
        if let None = self.values.remove_entry(&name.lexeme) {
            // If there's no existing entry, but we have a parent scope,
            // return the result from assign() in the parent scope.
            if let Some(enclosing) = self.enclosing.as_mut() {
                return enclosing.assign(name, value);
            } else {
                // Otherwise, return a runtime error, since we're reached
                // the global scope and still haven't found the variable key.
                return Err(RuntimeError::new(
                    name.clone(),
                    format!("Undefined variable '{}'.", name.lexeme),
                ));
            }
        }

        self.values.insert(name.lexeme.clone(), value);

        Ok(())
    }

    /// Whether we have an enclosing scope or not.
    fn is_global(&self) -> bool {
        self.enclosing.is_some()
    }

    /// If we do have an enclosing scope, this will get it. Be careful, because
    /// this will panic if the scope doesn't exist.
    fn take_enclosing_scope(mut self) -> Option<Box<EnvironmentBase>> {
        if let Some(env) = self.enclosing.take() {
            if let Ok(env_base) = env.downcast::<EnvironmentBase>() {
                return Some(env_base);
            }
        }
        None
    }
}

pub struct EnvironmentManager {
    // previous_environment: Rc<RwLock<Option<Environment>>>,
    current_environment: Rc<RwLock<Option<EnvironmentBase>>>,
}

impl EnvironmentManager {
    /// Create a new environment.
    pub fn new() -> Self {
        Self {
            // previous_environment: Rc::new(RwLock::new(None)),
            current_environment: Rc::new(RwLock::new(Some(EnvironmentBase::new_global()))),
        }
    }

    pub fn enter_new_scope(&self) {
        if let Ok(mut current) = self.current_environment.try_write() {
            let env_trait_obj: Box<dyn EnvironmentTrait> = Box::new(current.take().unwrap());
            let new_scope = EnvironmentBase::new_scoped(env_trait_obj);
            current.replace(new_scope);
        }
    }

    pub fn exit_current_scope(&self) {
        if let Ok(mut current) = self.current_environment.try_write() {
            let child_scope = current.take().unwrap();
            if let Some(parent_scope) = child_scope.take_enclosing_scope() {
                current.replace(Box::into_inner(parent_scope));
            }
        }
    }

    pub fn define(&self, name: &String, value: LoxObject) {
        if let Ok(mut current_environment) = self.current_environment.try_write() {
            current_environment.as_mut().map(|v| {
                v.define(name, value);
            });
        }
    }

    pub fn get(&self, name: &Token) -> Result<LoxObject, RuntimeError> {
        if let Ok(current_environment) = self.current_environment.try_read() {
            let env = current_environment.as_ref().unwrap();
            return env.get(name);
        }

        Err(RuntimeError::new(
            name.clone(),
            format!("[internal] Unable to get '{}'.", name.lexeme),
        ))
    }

    pub fn assign(&self, name: &Token, value: LoxObject) -> Result<(), RuntimeError> {
        if let Ok(mut current_environment) = self.current_environment.try_write() {
            let curr_env = current_environment.as_mut().unwrap();
            return curr_env.assign(name, value);
        }

        Err(RuntimeError::new(
            name.clone(),
            format!("[internal] Unable to assign '{}'.", name.lexeme),
        ))
    }

    // fn is_global(&self) -> bool {
    //     let mut is_global = false;
    //     if let Ok(maybe_current_environment) = self.current_environment.try_read() {
    //         maybe_current_environment.as_ref().map(|c| {
    //             is_global = c.is_global();
    //         });
    //     }

    //     is_global
    // }
}
