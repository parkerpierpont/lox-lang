use crate::environment::EnvironmentManager;
use crate::errors;
use crate::expr::{Expr, ExprVisitor, VisitorTarget};
use crate::object::{LoxBoolean, LoxNil, LoxNumber, LoxObject, LoxString};
use crate::runtime_error::RuntimeError;
use crate::stmt::{Statement, StmtVisitor, StmtVisitorTarget};
use crate::token::{Token, TokenLiteral};
use crate::token_type::TokenType;
use std::rc::Rc;

pub struct Interpreter {
    environment: EnvironmentManager,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: EnvironmentManager::new(),
        }
    }

    pub fn interpret(&self, statements: Vec<Statement>) {
        for stmt in statements {
            match self.execute(stmt) {
                Ok(_) => {}
                Err(runtime_error) => {
                    errors::runtime_error(runtime_error);
                    break;
                }
            }
        }
    }

    fn execute(&self, stmt: Statement) -> Result<(), RuntimeError> {
        stmt.accept(self)
    }

    fn execute_block(&self, statements: &Vec<Statement>) -> Result<(), RuntimeError> {
        self.environment.enter_new_scope();

        for statement in statements {
            if let Err(runtime_error) = self.execute(statement.clone()) {
                return Err(runtime_error);
            }
        }

        self.environment.exit_current_scope();

        Ok(())
    }

    // Sends the expression back through the visitor implementation
    fn evaluate(&self, expr: &Rc<dyn Expr>) -> Result<LoxObject, RuntimeError> {
        expr.accept(self)
    }

    fn check_number_operand<'a>(
        &self,
        operator: &Token,
        operand: &'a LoxObject,
    ) -> Result<&'a LoxObject, RuntimeError> {
        if operand.instance_name() == "Number" {
            Ok(operand)
        } else {
            Err(RuntimeError::new(
                operator.clone(),
                "Operand must be a number.",
            ))
        }
    }

    fn check_number_operands<'a, 'b>(
        &self,
        operator: &Token,
        operand_a: &'a LoxObject,
        operand_b: &'b LoxObject,
    ) -> Result<(&'a LoxObject, &'b LoxObject), RuntimeError> {
        match (
            self.check_number_operand(operator, operand_a),
            self.check_number_operand(operator, operand_b),
        ) {
            (Ok(operand_a), Ok(operand_b)) => Ok((operand_a, operand_b)),
            (Ok(_), Err(err)) => Err(err),
            (Err(err), Ok(_)) => Err(err),
            (Err(err), Err(_)) => Err(err),
        }
    }
}

impl ExprVisitor<Result<LoxObject, RuntimeError>> for &Interpreter {
    fn visit_binary_expr(&self, expr: &crate::expr::Binary) -> Result<LoxObject, RuntimeError> {
        let (left, right) = (self.evaluate(&expr.left), self.evaluate(&expr.right));
        if left.is_err() {
            return left;
        } else if right.is_err() {
            return right;
        }

        let lft = left.unwrap();
        let rgt = right.unwrap();
        let l_ty = lft.instance_name();
        let r_ty = rgt.instance_name();

        match expr.operator.ty {
            TokenType::Minus => match self.check_number_operands(&expr.operator, &lft, &rgt) {
                Ok((left, right)) => Ok(LoxNumber::new(left.get_number() - right.get_number())),
                Err(err) => Err(err),
            },
            TokenType::Plus => match self.check_number_operands(&expr.operator, &lft, &rgt) {
                Ok((left, right)) => Ok(LoxNumber::new(left.get_number() + right.get_number())),
                _ => {
                    if l_ty == "String" && r_ty == "String" {
                        Ok(LoxString::new(lft.get_string() + rgt.get_string().as_str()))
                    } else {
                        Err(RuntimeError::new(
                            expr.operator.clone(),
                            "Operands must both be numbers or strings.",
                        ))
                    }
                }
            },
            TokenType::Slash => match self.check_number_operands(&expr.operator, &lft, &rgt) {
                Ok((left, right)) => Ok(LoxNumber::new(left.get_number() / right.get_number())),
                Err(err) => Err(err),
            },
            TokenType::Star => match self.check_number_operands(&expr.operator, &lft, &rgt) {
                Ok((left, right)) => Ok(LoxNumber::new(left.get_number() * right.get_number())),
                Err(err) => Err(err),
            },
            TokenType::Greater => match self.check_number_operands(&expr.operator, &lft, &rgt) {
                Ok((left, right)) => Ok(LoxBoolean::new(left.get_number() > right.get_number())),
                Err(err) => Err(err),
            },
            TokenType::GreaterEqual => match self.check_number_operands(&expr.operator, &lft, &rgt)
            {
                Ok((left, right)) => Ok(LoxBoolean::new(left.get_number() >= right.get_number())),
                Err(err) => Err(err),
            },
            TokenType::Less => match self.check_number_operands(&expr.operator, &lft, &rgt) {
                Ok((left, right)) => Ok(LoxBoolean::new(left.get_number() < right.get_number())),
                Err(err) => Err(err),
            },
            TokenType::LessEqual => match self.check_number_operands(&expr.operator, &lft, &rgt) {
                Ok((left, right)) => Ok(LoxBoolean::new(left.get_number() <= right.get_number())),
                Err(err) => Err(err),
            },
            TokenType::BangEqual => Ok(LoxBoolean::new(lft != rgt)),
            TokenType::EqualEqual => Ok(LoxBoolean::new(lft == rgt)),
            _ => unreachable!(),
        }
    }

    fn visit_grouping_expr(&self, expr: &crate::expr::Grouping) -> Result<LoxObject, RuntimeError> {
        self.evaluate(&expr.expression)
    }

    fn visit_literal_expr(&self, expr: &crate::expr::Literal) -> Result<LoxObject, RuntimeError> {
        Ok(match &expr.value {
            TokenLiteral::String(value) => LoxString::new(value.clone()),
            TokenLiteral::Number(value) => LoxNumber::new(*value),
            TokenLiteral::False => LoxBoolean::new(false),
            TokenLiteral::True => LoxBoolean::new(true),
            TokenLiteral::None => LoxNil::new(),
        })
    }

    fn visit_unary_expr(&self, expr: &crate::expr::Unary) -> Result<LoxObject, RuntimeError> {
        self.evaluate(&expr.right)
            .map(|right| match expr.operator.ty {
                TokenType::Minus => {
                    if let "Number" = right.instance_name() {
                        return LoxNumber::new(-right.get_number());
                    }

                    right
                }
                TokenType::Bang => LoxBoolean::new(right.is_truthy()),
                _ => unreachable!(),
            })
    }

    fn visit_variable_expr(&self, expr: &crate::expr::Variable) -> Result<LoxObject, RuntimeError> {
        self.environment.get(&expr.name)
    }

    fn visit_assign_expr(&self, expr: &crate::expr::Assign) -> Result<LoxObject, RuntimeError> {
        let value = match self.evaluate(&expr.value) {
            Ok(val) => val,
            Err(runtime_error) => return Err(runtime_error),
        };

        if let Err(runtime_error) = self.environment.assign(&expr.name, value.clone()) {
            return Err(runtime_error);
        }

        return Ok(value);
    }
}

impl StmtVisitor<Result<(), RuntimeError>> for &Interpreter {
    fn visit_expression_stmt(&self, stmt: &crate::stmt::ExprStmt) -> Result<(), RuntimeError> {
        match self.evaluate(&stmt.expression) {
            Ok(_) => Ok(()),
            Err(runtime_error) => Err(runtime_error),
        }
    }

    fn visit_print_stmt(&self, stmt: &crate::stmt::PrintStmt) -> Result<(), RuntimeError> {
        let value = self.evaluate(&stmt.expression);
        match value {
            Ok(print_value) => {
                println!("{}", print_value.stringify());
                Ok(())
            }
            Err(runtime_error) => Err(runtime_error),
        }
    }

    fn visit_variable_stmt(&self, stmt: &crate::stmt::VariableStmt) -> Result<(), RuntimeError> {
        let value = match &stmt.initializer {
            // If we have an initializer, we need to evaluate the expression to
            // get the final value.
            Some(initializer) => match self.evaluate(initializer) {
                Ok(initializer_value) => initializer_value,
                Err(runtime_error) => {
                    return Err(runtime_error);
                }
            },
            // If we don't have an initializer, we set the variable equal to nil.
            None => LoxNil::new(),
        };

        self.environment.define(&stmt.name.lexeme, value);

        Ok(())
    }

    fn visit_block_stmt(&self, stmt: &crate::stmt::BlockStmt) -> Result<(), RuntimeError> {
        if let Err(runtime_error) = self.execute_block(&stmt.statements) {
            return Err(runtime_error);
        }

        Ok(())
    }
}
