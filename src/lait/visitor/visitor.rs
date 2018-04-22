use super::super::error::Response::Wrong;

use std::rc::Rc;



#[derive(Debug, Clone, PartialEq)]
pub enum Type {
  Int,
  Float,
  Char,
  String,
  Bool,
  Nil,
  Id(String),
}