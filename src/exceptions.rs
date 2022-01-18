use crate::{object::LoxObject, token::Token};

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub token: Token,
    pub message: String,
}

impl RuntimeError {
    pub fn new(token: Token, message: impl Into<String>) -> RuntimeException {
        RuntimeException::RuntimeError(Self {
            token,
            message: message.into(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct ReturnException {
    pub value: LoxObject,
}

impl ReturnException {
    pub fn new(value: LoxObject) -> RuntimeException {
        RuntimeException::ReturnException(Self { value })
    }
}

#[derive(Debug, Clone)]
pub enum RuntimeException {
    RuntimeError(RuntimeError),
    ReturnException(ReturnException),
}
