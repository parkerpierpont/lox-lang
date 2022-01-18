use crate::{
    exceptions::{RuntimeError, RuntimeException},
    object::LoxObject,
    token::Token,
};
use std::{collections::HashMap, rc::Rc, sync::RwLock};

pub trait EnvironmentTrait {
    fn define(&mut self, name: &String, value: LoxObject);
    fn get(&self, name: &Token) -> Option<LoxObject>;
    fn assign(&mut self, name: &Token, value: LoxObject) -> Option<()>;
}

pub struct EnvironmentBase {
    pub values: HashMap<String, LoxObject>,
}

impl EnvironmentBase {
    /// Create a new environment.
    pub fn new_global() -> EnvironmentBase {
        Self {
            values: HashMap::new(),
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
    fn get(&self, name: &Token) -> Option<LoxObject> {
        self.values.get(&name.lexeme).map(|v| v.clone())
    }

    /// Similar to 'get', but this doesn't let you create a new variable. If a
    /// new variable creation is attempted, this will throw a 'RuntimeError'.
    fn assign(&mut self, name: &Token, value: LoxObject) -> Option<()> {
        // If the key exists, replace it with new value.
        if let None = self.values.remove_entry(&name.lexeme) {
            return None;
        }

        self.values.insert(name.lexeme.clone(), value);

        Some(())
    }
}

pub struct EnvironmentStack {
    inner: Rc<RwLock<Vec<Rc<RwLock<EnvironmentBase>>>>>,
}

impl EnvironmentStack {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RwLock::new(vec![Rc::new(RwLock::new(
                EnvironmentBase::new_global(),
            ))])),
        }
    }

    pub fn enter_new_scope(&self) {
        if let Ok(mut inner) = self.inner.try_write() {
            inner.push(Rc::new(RwLock::new(EnvironmentBase::new_global())));
        }
    }

    pub fn exit_scope(&self) {
        if let Ok(mut inner) = self.inner.try_write() {
            let len = inner.len();
            if len > 1 as usize {
                inner.remove(len - 1);
            }
        }
    }

    pub fn define(&self, name: &String, value: LoxObject) {
        if let Ok(inner) = self.inner.try_read() {
            let v = inner.last().unwrap();
            if let Ok(mut v) = v.try_write() {
                v.define(name, value);
                return;
            }
        }

        panic!("Unable to define new value in [EnvironmentStack::define]");
    }

    pub fn get(&self, name: &Token) -> Result<LoxObject, RuntimeException> {
        let mut ret = None;
        if let Ok(inner) = self.inner.try_read() {
            let mut idx = inner.len() - 1;
            while idx >= 0 as usize && ret.is_none() {
                let v = inner.get(idx).unwrap();
                if let Ok(env) = v.try_write() {
                    match env.get(name) {
                        Some(value) => {
                            ret = Some(value);
                            break;
                        }
                        None => {
                            idx = idx - 1;
                        }
                    }
                }
            }
        }

        match ret {
            Some(obj) => Ok(obj),
            None => Err(RuntimeError::new(
                name.clone(),
                format!("[internal] Unable to get '{}'.", name.lexeme),
            )),
        }
    }

    pub fn assign(&self, name: &Token, value: LoxObject) -> Result<(), RuntimeException> {
        let mut ret = None;
        if let Ok(inner) = self.inner.try_read() {
            let mut idx = inner.len() - 1;

            while idx >= 0 as usize && ret.is_none() {
                let v = inner.get(idx).unwrap();
                if let Ok(mut env) = v.try_write() {
                    match env.assign(name, value.clone()) {
                        Some(_) => {
                            ret = Some(());
                            break;
                        }
                        None => {
                            idx = idx - 1;
                        }
                    }
                }
            }
        }

        match ret {
            Some(_) => Ok(()),
            None => Err(RuntimeError::new(
                name.clone(),
                format!("[internal] Unable to assign '{}'.", name.lexeme),
            )),
        }
    }

    pub fn new_from_current_global(&self) -> Self {
        let mut ret = None;
        if let Ok(inner) = self.inner.try_read() {
            ret = inner.get(0).map(|v| v.clone());
        }

        if let Some(global) = ret {
            Self {
                inner: Rc::new(RwLock::new(vec![global])),
            }
        } else {
            panic!("Unable to construct new EnvironmentStack from current global.")
        }
    }
}

pub struct EnvironmentManager {
    pub environments: Rc<RwLock<Vec<EnvironmentStack>>>,
}

impl EnvironmentManager {
    /// Create a new environment.
    pub fn new() -> Self {
        Self {
            environments: Rc::new(RwLock::new(vec![EnvironmentStack::new()])),
        }
    }

    /// Enters a new function scope
    pub fn enter_function_scope(&self) {
        if let Ok(mut environments) = self.environments.try_write() {
            let new_env = environments.get(0).unwrap().new_from_current_global();
            environments.push(new_env);
            return;
        }

        panic!("Unable to enter function scope.")
    }

    /// Exits the most recent function scope.
    pub fn exit_function_scope(&self) {
        let mut exited = false;
        if let Ok(mut environments) = self.environments.try_write() {
            let len = environments.len();
            if len > 1 {
                environments.remove(len - 1);
                exited = true;
            }
        }

        if !exited {
            panic!("Unable to exit function scope.")
        }
    }

    pub fn enter_new_scope(&self) {
        if let Ok(environments) = self.environments.try_read() {
            if let Some(environment_stack) = environments.last() {
                environment_stack.enter_new_scope();
            }
        }
    }

    pub fn exit_current_scope(&self) {
        if let Ok(environments) = self.environments.try_read() {
            if let Some(environment_stack) = environments.last() {
                environment_stack.exit_scope();
            }
        }
    }

    pub fn define(&self, name: &String, value: LoxObject) {
        if let Ok(environments) = self.environments.try_read() {
            if let Some(environment_stack) = environments.last() {
                environment_stack.define(name, value);
            }
        }
    }

    pub fn get(&self, name: &Token) -> Result<LoxObject, RuntimeException> {
        if let Ok(environments) = self.environments.try_read() {
            if let Some(environment_stack) = environments.last() {
                return environment_stack.get(name);
            }
        }

        Err(RuntimeError::new(
            name.clone(),
            format!("[internal] Unable to get '{}'.", name.lexeme),
        ))
    }

    pub fn assign(&self, name: &Token, value: LoxObject) -> Result<(), RuntimeException> {
        if let Ok(environments) = self.environments.try_read() {
            if let Some(environment_stack) = environments.last() {
                return environment_stack.assign(name, value);
            }
        }

        Err(RuntimeError::new(
            name.clone(),
            format!("[internal] Unable to assign '{}'.", name.lexeme),
        ))
    }
}
