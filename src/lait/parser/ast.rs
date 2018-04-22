use super::*;
use super::visitor::Type;

use std::rc::Rc;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum StatementNode<'s> {
  Expression(Expression<'s>),
  Variable(Type, Expression<'s>, Option<Expression<'s>>),
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
  Bool(bool),
  Identifier(String),
  Block(Vec<Statement<'e>>),
  Binary(Rc<Expression<'e>>, Operator, Rc<Expression<'e>>),
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



#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
  Add, Sub, Mul, Div, Mod, Pow, Concat, Eq, Lt, Gt, NEq, LtEq, GtEq,
}

impl Operator {
  pub fn from_str(operator: &str) -> Option<(Operator, u8)> {
    use self::Operator::*;

    let op_prec = match operator {
      "==" => (Eq,     0),
      "<"  => (Lt,     0),
      ">"  => (Gt,     0),
      "!=" => (NEq,    0),
      "<=" => (LtEq,   0),
      ">=" => (GtEq,   0),
      "+"  => (Add,    1),
      "-"  => (Sub,    1),
      "++" => (Concat, 1),
      "*"  => (Mul,    2),
      "/"  => (Div,    2),
      "%"  => (Mod,    2),
      "^"  => (Pow,    3),
      _    => return None,
    };

    Some(op_prec)
  }

  pub fn as_str(&self) -> &str {
    use self::Operator::*;
    
    match *self {
      Add    => "+",
      Sub    => "-",
      Concat => "++",
      Mul    => "*",
      Div    => "/",
      Mod    => "%",
      Pow    => "^",
      Eq     => "==",
      Lt     => "<",
      Gt     => ">",
      NEq    => "!=",
      LtEq   => "<=",
      GtEq   => ">=",
    }
  }
}

impl fmt::Display for Operator {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.as_str())
  }
}