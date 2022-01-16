use lazy_static::lazy_static;
use std::sync::{atomic::AtomicBool, Arc, RwLock};

use crate::{runtime_error::RuntimeError, token::Token};

#[derive(Debug, Clone)]

struct Error {
    line: usize,
    _where: String,
    msg: String,
}

#[derive(Debug)]
struct ErrorManager {
    errors: Arc<RwLock<Vec<Error>>>,
    immediate: AtomicBool,
    had_errors: AtomicBool,
    had_runtime_error: AtomicBool,
}

impl ErrorManager {
    pub fn new() -> Self {
        Self {
            errors: Arc::new(RwLock::new(Vec::new())),
            immediate: AtomicBool::new(false),
            had_errors: AtomicBool::new(false),
            had_runtime_error: AtomicBool::new(false),
        }
    }

    pub fn set_immediate(&self, immediate: bool) {
        self.immediate
            .store(immediate, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn error(&self, line: usize, message: String) {
        self.had_errors
            .store(true, std::sync::atomic::Ordering::SeqCst);
        if self.immediate.load(std::sync::atomic::Ordering::SeqCst) {
            return Self::display_error(line, "".to_string(), message);
        }

        if let Ok(mut writable) = self.errors.try_write() {
            writable.push(Error {
                line,
                _where: "".to_string(),
                msg: message,
            })
        }
    }

    pub fn runtime_error(&self, token: Token, message: String) {
        self.had_runtime_error
            .store(true, std::sync::atomic::Ordering::SeqCst);
        if self.immediate.load(std::sync::atomic::Ordering::SeqCst) {
            return Self::display_error(
                token.line,
                "RuntimeError(".to_string() + token.lexeme.as_str() + ")",
                message,
            );
        }

        if let Ok(mut writable) = self.errors.try_write() {
            writable.push(Error {
                line: token.line,
                _where: "(".to_string() + token.lexeme.as_str() + ")",
                msg: message,
            })
        }
    }

    pub fn report(&self, line: usize, _where: String, message: String) {
        self.had_errors
            .store(true, std::sync::atomic::Ordering::SeqCst);
        if self.immediate.load(std::sync::atomic::Ordering::SeqCst) {
            return Self::display_error(line, _where, message);
        }

        if let Ok(mut writable) = self.errors.try_write() {
            writable.push(Error {
                line,
                _where,
                msg: message,
            })
        }
    }

    fn display_error(line: usize, _where: String, message: String) {
        println!("[line {}] Error {}: {}", line, _where, message);
    }

    pub fn reset_had_errors(&self) {
        self.had_errors
            .store(false, std::sync::atomic::Ordering::SeqCst);
        self.had_runtime_error
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn print_all(&self) {
        if let Ok(readable) = self.errors.try_read() {
            if readable.is_empty() {
                return;
            }
            println!("Found {:?} errors:", readable.len());
            for Error { line, _where, msg } in readable.iter() {
                Self::display_error(*line, _where.clone(), msg.clone());
            }
        }
    }
}

unsafe impl Sync for ErrorManager {}

lazy_static! {
    /// This is an example for using doc comment attributes
    static ref ERROR_MANAGER: ErrorManager = {
        ErrorManager::new()
    };
}

pub fn initialize_immediate() {
    ERROR_MANAGER.set_immediate(true);
}

pub fn initialize_managed() {}

pub fn error(line: usize, message: impl Into<String>) {
    ERROR_MANAGER.error(line, message.into());
}

pub fn runtime_error(error: RuntimeError) {
    ERROR_MANAGER.runtime_error(error.token, error.message);
}

pub fn report(line: usize, _where: impl Into<String>, message: impl Into<String>) {
    ERROR_MANAGER.report(line, _where.into(), message.into());
}

pub fn print_all() {
    if !ERROR_MANAGER
        .immediate
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        ERROR_MANAGER.print_all();
    }
}

pub fn reset_errors() {
    ERROR_MANAGER.reset_had_errors();
}

pub fn has_errors() -> bool {
    ERROR_MANAGER
        .had_errors
        .load(std::sync::atomic::Ordering::SeqCst)
}

pub fn has_runtime_error() -> bool {
    ERROR_MANAGER
        .had_runtime_error
        .load(std::sync::atomic::Ordering::SeqCst)
}
