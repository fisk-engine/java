use super::super::error::Response::Wrong;

use std::rc::Rc;
use std::fmt;

use super::*;


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

impl Type {
  pub fn check_expression(&self, other: &ExpressionNode) -> bool {
    use self::Type::*;

    match *other {
      ExpressionNode::Int(_) => match *self {
        Int | Float => true,
        _           => false,
      },

      _ => false
    }
  }
}

impl fmt::Display for Type {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use self::Type::*;

    match *self {
      Int        => write!(f, "int"),
      Float      => write!(f, "float"),
      Char       => write!(f, "char"),
      String     => write!(f, "str"),
      Bool       => write!(f, "bool"),
      Nil        => write!(f, "nil"),
      Id(ref id) => write!(f, "{}", id),
    }
  }
}



pub struct Visitor<'v> {
  pub tabs:       Vec<(SymTab, TypeTab)>,
  pub tab_frames: Vec<(SymTab, TypeTab)>,

  pub source: &'v Source,
  pub ast:    &'v Vec<Statement<'v>>,

  pub depth: u32,
}

impl<'v> Visitor<'v> {
  pub fn new(source: &'v Source, ast: &'v Vec<Statement<'v>>) -> Self {
    Visitor {
      tabs:       vec!((SymTab::global(), TypeTab::global())),
      tab_frames: Vec::new(),

      source,
      ast,
      depth:   0,
    }
  }

  pub fn visit(&mut self) -> Result<(), ()> {
    for statement in self.ast {
      self.visit_statement(&statement)?
    }

    self.tab_frames.push(self.tabs.last().unwrap().clone());

    Ok(())
  }

  pub fn visit_statement(&mut self, statement: &'v Statement<'v>) -> Result<(), ()> {
    use self::StatementNode::*;

    match statement.node {
      Expression(ref expression) => self.visit_expression(expression),

      Variable(_, ref left, _) => match left.node {
        ExpressionNode::Identifier(_) => {
          self.visit_variable(&statement.node)
        },

        _ => Ok(())
      },
    }
  }

  fn visit_expression(&mut self, expression: &'v Expression<'v>) -> Result<(), ()> {
    use self::ExpressionNode::*;

    match expression.node {
      Identifier(ref name) => if self.current_tab().0.get_name(name).is_none() {
        Err(
          response!(
            Wrong(format!("no such value `{}` in this scope", name)),
            self.source.file,
            expression.pos
          )
        )
      } else {
        Ok(())
      },

      _ => Ok(()),
    }
  }



  fn visit_variable(&mut self, variable: &'v StatementNode) -> Result<(), ()> {
    use self::ExpressionNode::Identifier;

    if let &StatementNode::Variable(ref variable_type, ref left, ref right) = variable {
      match left.node {
        Identifier(ref name) => {
          let index = if let Some((index, _)) = self.current_tab().0.get_name(name) {
            index
          } else {
            self.current_tab().0.add_name(name)
          };

          self.current_tab().1.grow();

          if let &Some(ref right) = right {
            self.visit_expression(&right)?;

            let right_type = self.type_expression(&right)?;

            if *variable_type != Type::Nil {
              if !variable_type.check_expression(&Parser::fold_expression(right)?.node) && *variable_type != right_type {
                return Err(
                  response!(
                    Wrong(format!("mismatched types, expected type `{}` got `{}`", variable_type, right_type)),
                    self.source.file,
                    right.pos
                  )
                )
              } else {
                let depth  = self.depth;
                
                self.current_tab().1.set_type(index, 0, (variable_type.to_owned(), depth))?;
              }

            } else {
              let depth  = self.depth;
              
              self.current_tab().1.set_type(index, 0, (right_type, depth))?;
            }

          } else {
            let depth = self.depth;

            self.current_tab().1.set_type(index, 0, (variable_type.to_owned(), depth))?;
          }

          Ok(())
        },

        _ => return Err(
          response!(
            Wrong("unexpected variable declaration"),
            self.source.file,
            left.pos
          )
        )
      }
    } else {
      unreachable!()
    }
  }



  pub fn type_expression(&mut self, expression: &'v Expression<'v>) -> Result<Type, ()> {
    use self::ExpressionNode::*;

    let t = match expression.node {
      Identifier(ref name) => if let Some((index, env_index)) = self.current_tab().0.get_name(name) {
        self.current_tab().1.get_type(index, env_index)?
      } else {
        return Err(
          response!(
            Wrong(format!("no such value `{}` in this scope", name)),
            self.source.file,
            expression.pos
          )
        )
      },

      String(_) => Type::String,
      Char(_)   => Type::Char,
      Bool(_)   => Type::Bool,
      Int(_)    => Type::Int,
      Float(_)  => Type::Float,

      _ => Type::Nil,
    };

    Ok(t)
  }



  pub fn current_tab(&mut self) -> &mut (SymTab, TypeTab) {
    let len = self.tabs.len() - 1;

    &mut self.tabs[len]
  }



  pub fn push_scope(&mut self) {
    let local_symtab  = SymTab::new(Rc::new(self.current_tab().0.clone()), &[]);
    let local_typetab = TypeTab::new(Rc::new(self.current_tab().1.clone()), &[]);

    self.tabs.push((local_symtab.clone(), local_typetab.clone()));

    self.depth += 1
  }

  pub fn pop_scope(&mut self) {
    self.tab_frames.push(self.tabs.pop().unwrap());

    self.depth -= 1
  }
}