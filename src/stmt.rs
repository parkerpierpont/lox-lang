use std::{fmt::Debug, rc::Rc};

use crate::{expr::Expression, shared_traits::Named, token::Token};
use downcast::{downcast, Any};

pub trait Stmt: Any + Debug + Named {}

pub trait StmtVisitor<T> {
    fn visit_expression_stmt(&self, stmt: &ExprStmt) -> T;
    fn visit_print_stmt(&self, stmt: &PrintStmt) -> T;
    fn visit_variable_stmt(&self, stmt: &VariableStmt) -> T;
    fn visit_block_stmt(&self, stmt: &BlockStmt) -> T;
}

pub trait StmtVisitorTarget {
    fn accept<T>(&self, visitor: impl StmtVisitor<T>) -> T;
}

impl StmtVisitorTarget for Rc<dyn Stmt> {
    fn accept<T>(&self, visitor: impl StmtVisitor<T>) -> T {
        match self.name() {
            "Expression" => visitor.visit_expression_stmt(self.downcast_ref::<ExprStmt>().unwrap()),
            "Print" => visitor.visit_print_stmt(self.downcast_ref::<PrintStmt>().unwrap()),
            "Variable" => visitor.visit_variable_stmt(self.downcast_ref::<VariableStmt>().unwrap()),
            "Block" => visitor.visit_block_stmt(self.downcast_ref::<BlockStmt>().unwrap()),
            _ => unreachable!(),
        }
    }
}

downcast!(dyn Stmt);

pub type Statement = Rc<dyn Stmt>;

#[derive(Debug, Clone)]
pub struct ExprStmt {
    pub expression: Expression,
}

impl ExprStmt {
    pub fn new(expression: Expression) -> Statement {
        Rc::new(ExprStmt { expression })
    }
}
impl Stmt for ExprStmt {}
impl Named for ExprStmt {
    fn name(&self) -> &'static str {
        "Expression"
    }
}

#[derive(Debug, Clone)]
pub struct PrintStmt {
    pub expression: Expression,
}

impl PrintStmt {
    pub fn new(expression: Expression) -> Statement {
        Rc::new(PrintStmt { expression })
    }
}
impl Stmt for PrintStmt {}
impl Named for PrintStmt {
    fn name(&self) -> &'static str {
        "Print"
    }
}

#[derive(Debug, Clone)]
pub struct VariableStmt {
    pub name: Token,
    pub initializer: Option<Expression>,
}

impl VariableStmt {
    pub fn new(name: Token, initializer: Option<Expression>) -> Statement {
        Rc::new(VariableStmt { name, initializer })
    }
}
impl Stmt for VariableStmt {}
impl Named for VariableStmt {
    fn name(&self) -> &'static str {
        "Variable"
    }
}

#[derive(Debug, Clone)]
pub struct BlockStmt {
    pub statements: Vec<Statement>,
}

impl BlockStmt {
    pub fn new(statements: Vec<Statement>) -> Statement {
        Rc::new(BlockStmt { statements })
    }
}
impl Stmt for BlockStmt {}
impl Named for BlockStmt {
    fn name(&self) -> &'static str {
        "Block"
    }
}
