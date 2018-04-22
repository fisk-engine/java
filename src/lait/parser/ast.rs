use std::rc::Rc;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum StatementNode<'s> {
  Expression(Expression<'s>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Statement<'s> {
  pub node: StatementNode<'s>,
  pub pos:  TokenElement<'s>,
}

impl<'s> Statement<'s> {
  pub fn new(node: StatementNode<'s>, pos: TokenElement<'s>) -> Self {
    Statement {
      node,
      pos,
    }
  }
}



#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionNode<'e> {
  Int(u128),
  Float(f64),
  String(String),
  Char(char),
  Identifier(String),
  EOF,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expression<'e> {
  pub node: ExpressionNode<'e>,
  pub pos:  TokenElement<'e>,
}

impl<'e> Expression<'e> {
  pub fn new(node: ExpressionNode<'e>, pos: TokenElement<'e>) -> Self {
    Expression {
      node,
      pos,
    }
  }
}