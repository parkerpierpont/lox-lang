use crate::environment::EnvironmentManager;
use crate::errors;
use crate::exceptions::{ReturnException, RuntimeError, RuntimeException};
use crate::expr::{Expr, ExprVisitor, Literal, VisitorTarget};
use crate::function::{LoxFunction, LoxNativeCallable};
use crate::object::{LoxBoolean, LoxNil, LoxNumber, LoxObject, LoxString};
use crate::stmt::{Statement, StmtVisitor, StmtVisitorTarget};
use crate::token::{Token, TokenLiteral};
use crate::token_type::TokenType;
use std::rc::Rc;

pub struct Interpreter {
    pub environment: EnvironmentManager,
}

impl Interpreter {
    pub fn new() -> Self {
        let environment = EnvironmentManager::new();
        // Add native clock function
        environment.define(
            &"clock".to_string(),
            LoxNativeCallable::new(0, native_clock),
        );

        Self { environment }
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

    pub fn execute(&self, stmt: Statement) -> Result<(), RuntimeException> {
        stmt.accept(self)
    }

    pub fn execute_block(&self, statements: &Vec<Statement>) -> Result<(), RuntimeException> {
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
    pub fn evaluate(&self, expr: &Rc<dyn Expr>) -> Result<LoxObject, RuntimeException> {
        expr.accept(self)
    }

    pub fn check_number_operand<'a>(
        &self,
        operator: &Token,
        operand: &'a LoxObject,
    ) -> Result<&'a LoxObject, RuntimeException> {
        if operand.instance_name() == "Number" {
            Ok(operand)
        } else {
            Err(RuntimeError::new(
                operator.clone(),
                "Operand must be a number.",
            ))
        }
    }

    pub fn check_number_operands<'a, 'b>(
        &self,
        operator: &Token,
        operand_a: &'a LoxObject,
        operand_b: &'b LoxObject,
    ) -> Result<(&'a LoxObject, &'b LoxObject), RuntimeException> {
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

impl ExprVisitor<Result<LoxObject, RuntimeException>> for &Interpreter {
    fn visit_binary_expr(&self, expr: &crate::expr::Binary) -> Result<LoxObject, RuntimeException> {
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

    fn visit_grouping_expr(
        &self,
        expr: &crate::expr::Grouping,
    ) -> Result<LoxObject, RuntimeException> {
        self.evaluate(&expr.expression)
    }

    fn visit_literal_expr(
        &self,
        expr: &crate::expr::Literal,
    ) -> Result<LoxObject, RuntimeException> {
        Ok(match &expr.value {
            TokenLiteral::String(value) => LoxString::new(value.clone()),
            TokenLiteral::Number(value) => LoxNumber::new(*value),
            TokenLiteral::False => LoxBoolean::new(false),
            TokenLiteral::True => LoxBoolean::new(true),
            TokenLiteral::None => LoxNil::new(),
        })
    }

    fn visit_unary_expr(&self, expr: &crate::expr::Unary) -> Result<LoxObject, RuntimeException> {
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

    fn visit_variable_expr(
        &self,
        expr: &crate::expr::Variable,
    ) -> Result<LoxObject, RuntimeException> {
        self.environment.get(&expr.name)
    }

    fn visit_assign_expr(&self, expr: &crate::expr::Assign) -> Result<LoxObject, RuntimeException> {
        let value = match self.evaluate(&expr.value) {
            Ok(val) => val,
            Err(runtime_error) => return Err(runtime_error),
        };

        if let Err(runtime_error) = self.environment.assign(&expr.name, value.clone()) {
            return Err(runtime_error);
        }

        return Ok(value);
    }

    fn visit_logical_expr(
        &self,
        expr: &crate::expr::Logical,
    ) -> Result<LoxObject, RuntimeException> {
        let left = match self.evaluate(&expr.left) {
            Ok(obj) => obj,
            Err(runtime_error) => return Err(runtime_error),
        };

        if expr.operator.ty == TokenType::Or {
            // "OR" short circut – we know one of the values is truthy.
            if left.is_truthy() {
                return Ok(left);
            }
        } else {
            // "AND" short_circuit - we know one of the values isn't truthy.
            if !left.is_truthy() {
                return Ok(left);
            }
        }

        // If no short-circuit, we have to rely upon the final value of the
        // right-side of the expression.
        self.evaluate(&expr.right)
    }

    fn visit_call_expr(&self, expr: &crate::expr::Call) -> Result<LoxObject, RuntimeException> {
        let callee = match self.evaluate(&expr.callee) {
            Ok(callee_obj) => callee_obj,
            Err(runtime_error) => return Err(runtime_error),
        };

        let mut arguments = vec![];
        for argument in &expr.arguments {
            match self.evaluate(&argument) {
                Ok(argument_obj) => arguments.push(argument_obj),
                Err(runtime_error) => return Err(runtime_error),
            };
        }

        if !callee.is_callable() {
            return Err(RuntimeError::new(
                expr.paren.clone(),
                "Can only call functions and classes.",
            ));
        }

        let function = callee;

        if arguments.len() != function.arity() {
            return Err(RuntimeError::new(
                expr.paren.clone(),
                format!(
                    "Expected {} arguments but got {}.",
                    function.arity(),
                    arguments.len()
                ),
            ));
        }

        function.call(&self, arguments)
    }
}

impl StmtVisitor<Result<(), RuntimeException>> for &Interpreter {
    fn visit_expression_stmt(&self, stmt: &crate::stmt::ExprStmt) -> Result<(), RuntimeException> {
        match self.evaluate(&stmt.expression) {
            Ok(_) => Ok(()),
            Err(runtime_error) => Err(runtime_error),
        }
    }

    fn visit_print_stmt(&self, stmt: &crate::stmt::PrintStmt) -> Result<(), RuntimeException> {
        let value = self.evaluate(&stmt.expression);
        match value {
            Ok(print_value) => {
                println!("{}", print_value.stringify());
                Ok(())
            }
            Err(runtime_error) => Err(runtime_error),
        }
    }

    fn visit_variable_stmt(
        &self,
        stmt: &crate::stmt::VariableStmt,
    ) -> Result<(), RuntimeException> {
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

    fn visit_block_stmt(&self, stmt: &crate::stmt::BlockStmt) -> Result<(), RuntimeException> {
        if let Err(runtime_error) = self.execute_block(&stmt.statements) {
            return Err(runtime_error);
        }

        Ok(())
    }

    fn visit_if_stmt(&self, stmt: &crate::stmt::IfStmt) -> Result<(), RuntimeException> {
        let condition = match self.evaluate(&stmt.condition) {
            Ok(res) => res.is_truthy(),
            Err(runtime_error) => return Err(runtime_error),
        };

        match condition {
            true => match self.execute(stmt.then_branch.clone()) {
                Err(runtime_error) => Err(runtime_error),
                _ => Ok(()),
            },
            false => match stmt.else_branch.clone() {
                Some(stmt) => match self.execute(stmt) {
                    Err(runtime_error) => Err(runtime_error),
                    _ => Ok(()),
                },
                None => Ok(()),
            },
        }
    }

    fn visit_while_stmt(&self, stmt: &crate::stmt::WhileStmt) -> Result<(), RuntimeException> {
        while match self.evaluate(&stmt.condition) {
            // This is our evaluation of conditional's truthiness
            Ok(condition) => condition.is_truthy(),
            // If we can't evaluate the truthiness of the condition, we'll return.
            Err(runtime_error) => return Err(runtime_error),
        } {
            match self.execute(stmt.body.clone()) {
                Err(runtime_error) => return Err(runtime_error),
                _ => {}
            }
        }

        Ok(())
    }

    fn visit_fun_stmt(&self, stmt: &crate::stmt::FunStmt) -> Result<(), RuntimeException> {
        let function = LoxFunction::new(stmt);
        self.environment.define(&stmt.name.lexeme, function);
        Ok(())
    }

    fn visit_return_stmt(&self, stmt: &crate::stmt::ReturnStmt) -> Result<(), RuntimeException> {
        let is_null = stmt.value.name() == "Literal"
            && stmt.value.clone().downcast_rc::<Literal>().unwrap().value == TokenLiteral::None;

        let value = if !is_null {
            match self.evaluate(&stmt.value) {
                // The normal lox object.
                Ok(lox_obj) => lox_obj,
                // We have an actual runtime error here.
                Err(RuntimeException::RuntimeError(err)) => {
                    return Err(RuntimeException::RuntimeError(err))
                }
                // Shouldn't be possible to have a return statement inside of
                // another return statement.
                _ => {
                    return Err(RuntimeError::new(
                        stmt.keyword.clone(),
                        "Cannot use nested return values.",
                    ))
                }
            }
        } else {
            LoxNil::new()
        };

        // This is the successful code path, but we have to wrap it with an
        // exception so we can unwind.
        Err(ReturnException::new(value))
    }
}

/// Native Clock Function
fn native_clock(
    _interpreter: &Interpreter,
    _args: Vec<LoxObject>,
) -> Result<LoxObject, RuntimeException> {
    Ok(LoxNumber::new(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64(),
    ))
}
