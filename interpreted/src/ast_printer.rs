// use std::rc::Rc;

// use crate::{
//     expr::{Binary, Expr, ExprVisitor, Grouping, Literal, Unary, VisitorTarget},
//     token::TokenLiteral,
// };

// pub struct AstPrinter;

// impl AstPrinter {
//     fn parenthesize(&self, name: impl Into<String>, exprs: &[&Rc<dyn Expr>]) -> String {
//         let mut s = "".to_string();
//         for expr in exprs {
//             s = s + " ";
//             s = s + expr.accept(self).as_str();
//         }
//         format!("({}{})", name.into(), s)
//     }
// }

// impl ExprVisitor<String> for &AstPrinter {
//     fn visit_binary_expr(&self, expr: &Binary) -> String {
//         self.parenthesize(expr.operator.lexeme.clone(), &[&expr.left, &expr.right])
//     }

//     fn visit_grouping_expr(&self, expr: &Grouping) -> String {
//         self.parenthesize("group", &[&expr.expression])
//     }

//     fn visit_literal_expr(&self, expr: &Literal) -> String {
//         if expr.value == TokenLiteral::None {
//             return "nil".to_string();
//         }
//         return expr.value.to_string();
//     }

//     fn visit_unary_expr(&self, expr: &Unary) -> String {
//         self.parenthesize(expr.operator.lexeme.clone(), &[&expr.right])
//     }
// }

// #[cfg(test)]
// mod tests {
//     use crate::ast_printer::AstPrinter;
//     use crate::expr::Expr;
//     use crate::expr::{self, VisitorTarget};
//     use crate::token;
//     use crate::token::TokenLiteral;
//     use crate::token_type::TokenType;
//     use std::rc::Rc;

//     #[test]
//     pub fn test_ast_printer() {
//         let expression: Rc<dyn Expr> = Rc::new(expr::Binary {
//             left: Rc::new(expr::Unary {
//                 operator: token::Token {
//                     ty: TokenType::Minus,
//                     lexeme: "-".to_string(),
//                     literal: TokenLiteral::None,
//                     line: 1,
//                 },
//                 right: Rc::new(expr::Literal {
//                     value: TokenLiteral::Number(123.0),
//                 }),
//             }),
//             operator: token::Token {
//                 ty: TokenType::Star,
//                 lexeme: "*".to_string(),
//                 literal: token::TokenLiteral::None,
//                 line: 1,
//             },
//             right: Rc::new(expr::Grouping {
//                 expression: Rc::new(expr::Literal {
//                     value: TokenLiteral::Number(45.62),
//                 }),
//             }),
//         });

//         let printer = AstPrinter;
//         // printer.v
//         assert_eq!("(* (- 123) (group 45.62))", expression.accept(&printer))
//     }
// }
