use std::{fmt::Debug, rc::Rc};

use crate::{
    shared_traits::Named,
    token::{Token, TokenLiteral},
};
use downcast::{downcast, Any};

pub trait Expr: Any + Debug + Named {}

pub trait ExprVisitor<T> {
    fn visit_binary_expr(&self, expr: &Binary) -> T;
    fn visit_grouping_expr(&self, expr: &Grouping) -> T;
    fn visit_literal_expr(&self, expr: &Literal) -> T;
    fn visit_unary_expr(&self, expr: &Unary) -> T;
    fn visit_variable_expr(&self, expr: &Variable) -> T;
    fn visit_assign_expr(&self, expr: &Assign) -> T;
    fn visit_logical_expr(&self, expr: &Logical) -> T;
    fn visit_call_expr(&self, expr: &Call) -> T;
}

pub trait VisitorTarget {
    fn accept<T>(&self, visitor: impl ExprVisitor<T>) -> T;
}

impl VisitorTarget for Rc<dyn Expr> {
    fn accept<T>(&self, visitor: impl ExprVisitor<T>) -> T {
        match self.name() {
            "Binary" => visitor.visit_binary_expr(self.downcast_ref::<Binary>().unwrap()),
            "Grouping" => visitor.visit_grouping_expr(self.downcast_ref::<Grouping>().unwrap()),
            "Literal" => visitor.visit_literal_expr(self.downcast_ref::<Literal>().unwrap()),
            "Logical" => visitor.visit_logical_expr(self.downcast_ref::<Logical>().unwrap()),
            "Unary" => visitor.visit_unary_expr(self.downcast_ref::<Unary>().unwrap()),
            "Variable" => visitor.visit_variable_expr(self.downcast_ref::<Variable>().unwrap()),
            "Assign" => visitor.visit_assign_expr(self.downcast_ref::<Assign>().unwrap()),
            "Call" => visitor.visit_call_expr(self.downcast_ref::<Call>().unwrap()),
            _ => unreachable!(),
        }
    }
}

downcast!(dyn Expr);

pub type Expression = Rc<dyn Expr>;

#[derive(Debug, Clone)]
pub struct Binary {
    pub left: Expression,
    pub operator: Token,
    pub right: Expression,
}

impl Binary {
    pub fn new(left: Expression, operator: Token, right: Expression) -> Expression {
        Rc::new(Binary {
            left,
            operator,
            right,
        })
    }
}
impl Expr for Binary {}
impl Named for Binary {
    fn name(&self) -> &'static str {
        "Binary"
    }
}

#[derive(Debug, Clone)]
pub struct Grouping {
    pub expression: Expression,
}
impl Grouping {
    pub fn new(expression: Expression) -> Expression {
        Rc::new(Grouping { expression })
    }
}
impl Expr for Grouping {}
impl Named for Grouping {
    fn name(&self) -> &'static str {
        "Grouping"
    }
}

#[derive(Debug, Clone)]
pub struct Literal {
    pub value: TokenLiteral,
}
impl Literal {
    pub fn new(value: TokenLiteral) -> Expression {
        Rc::new(Literal { value })
    }
}
impl Expr for Literal {}
impl Named for Literal {
    fn name(&self) -> &'static str {
        "Literal"
    }
}

#[derive(Debug, Clone)]
pub struct Unary {
    pub operator: Token,
    pub right: Expression,
}
impl Unary {
    pub fn new(operator: Token, right: Expression) -> Expression {
        Rc::new(Unary { operator, right })
    }
}
impl Expr for Unary {}
impl Named for Unary {
    fn name(&self) -> &'static str {
        "Unary"
    }
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: Token,
}
impl Variable {
    pub fn new(name: Token) -> Expression {
        Rc::new(Variable { name })
    }
}
impl Expr for Variable {}
impl Named for Variable {
    fn name(&self) -> &'static str {
        "Variable"
    }
}

#[derive(Debug, Clone)]
pub struct Assign {
    pub name: Token,
    pub value: Expression,
}
impl Assign {
    pub fn new(name: Token, value: Expression) -> Expression {
        Rc::new(Assign { name, value })
    }
}
impl Expr for Assign {}
impl Named for Assign {
    fn name(&self) -> &'static str {
        "Assign"
    }
}

#[derive(Debug, Clone)]
pub struct Logical {
    pub left: Expression,
    pub operator: Token,
    pub right: Expression,
}
impl Logical {
    pub fn new(left: Expression, operator: Token, right: Expression) -> Expression {
        Rc::new(Logical {
            left,
            operator,
            right,
        })
    }
}
impl Expr for Logical {}
impl Named for Logical {
    fn name(&self) -> &'static str {
        "Logical"
    }
}

#[derive(Debug, Clone)]
pub struct Call {
    pub callee: Expression,
    pub paren: Token,
    pub arguments: Vec<Expression>,
}

impl Call {
    pub fn new(callee: Expression, paren: Token, arguments: Vec<Expression>) -> Expression {
        Rc::new(Call {
            callee,
            paren,
            arguments,
        })
    }
}
impl Expr for Call {}
impl Named for Call {
    fn name(&self) -> &'static str {
        "Call"
    }
}
