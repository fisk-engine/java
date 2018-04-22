use super::*;
use super::visitor::Type;
use super::super::error::Response::Wrong;

use std::rc::Rc;

pub struct Parser<'p> {
  index:  usize,
  tokens: Vec<&'p Token<'p>>,
  source: &'p Source,
}

impl<'p> Parser<'p> {
  pub fn new(tokens: Vec<&'p Token<'p>>, source: &'p Source) -> Self {
    Parser {
      tokens,
      source,
      index: 0,
    }
  }



  pub fn parse(&mut self) -> Result<Vec<Statement<'p>>, ()> {
    let mut ast = Vec::new();

    while self.remaining() > 0 {
      ast.push(self.parse_statement()?)
    }

    Ok(ast)
  }

  fn parse_statement(&mut self) -> Result<Statement<'p>, ()> {
    use self::TokenType::*;

    while self.current_type() == &EOL && self.remaining() != 0 {
      self.next()?
    }

    let statement = match *self.current_type() {
      _ => {
        use self::ExpressionNode::*;

        let expression = self.parse_expression()?;

        match expression.node {
          Identifier(_) => {
            if self.remaining() > 0 {
              if self.current_type() == &TokenType::Symbol {
                let statement = match self.current_lexeme().as_str() {
                  ":"   => self.parse_declaration(expression)?,
                  ref c => return Err(
                    response!(
                      Wrong(format!("unexpected symbol `{}`", c)),
                      self.source.file,
                      TokenElement::Ref(self.current())
                    )
                  )
                };

                statement
              } else {
                let position = expression.pos.clone();

                Statement::new(
                  StatementNode::Expression(expression),
                  position
                )
              }
            } else {
              let position = expression.pos.clone();

              Statement::new(
                StatementNode::Expression(expression),
                position,
              )
            }
          },

          _ => {
            let position = expression.pos.clone();

            Statement::new(
              StatementNode::Expression(expression),
              position,
            )
          },
        }
      },
    };

    Ok(statement)
  }

  pub fn parse_expression(&mut self) -> Result<Expression<'p>, ()> {
    let atom = self.parse_atom()?;

    if self.current_type() == &TokenType::Operator {
      self.parse_binary(atom)
    } else {
      Ok(atom)
    }
  }

  fn parse_atom(&mut self) -> Result<Expression<'p>, ()> {
    use self::TokenType::*;

    if self.remaining() == 0 {
      Ok(
        Expression::new(
          ExpressionNode::EOF,
          self.current_position()
        )
      )
    } else {
      let token_type = self.current_type().clone();
      let position   = self.current_position();

      let expression = match token_type {
        Int => Expression::new(
          ExpressionNode::Int(self.eat()?.parse::<u128>().unwrap()),
          position
        ),

        Float => Expression::new(
          ExpressionNode::Float(self.eat()?.parse::<f64>().unwrap()),
          position
        ),

        Char => Expression::new(
          ExpressionNode::Char(self.eat()?.chars().last().unwrap()),
          position
        ),

        String => Expression::new(
          ExpressionNode::String(self.eat()?),
          position
        ),

        Identifier => Expression::new(
          ExpressionNode::Identifier(self.eat()?),
          position
        ),

        Bool => Expression::new(
          ExpressionNode::Bool(self.eat()? == "true"),
          position
        ),

        _ => return Err(
          response!(
            Wrong("unimplemented af"),
            self.source.file,
            self.span_from(position)
          )
        )
      };

      Ok(expression)
    }
  }

  fn parse_binary(&mut self, left: Expression<'p>) -> Result<Expression<'p>, ()> {
    let left_position = left.pos.clone();

    let mut expression_stack = vec!(left);
    let mut operator_stack   = vec!(Operator::from_str(&self.eat()?).unwrap());

    expression_stack.push(self.parse_atom()?);

    while operator_stack.len() > 0 {
      while self.current_type() == &TokenType::Operator {
        let position               = self.current_position();
        let (operator, precedence) = Operator::from_str(&self.eat()?).unwrap();

        if precedence < operator_stack.last().unwrap().1 {
          let right = expression_stack.pop().unwrap();
          let left  = expression_stack.pop().unwrap();

          expression_stack.push(
            Expression::new(
              ExpressionNode::Binary(Rc::new(left), operator_stack.pop().unwrap().0, Rc::new(right)),
              self.current_position(),
            )
          );

          if self.remaining() > 0 {
            expression_stack.push(self.parse_atom()?);
            operator_stack.push((operator, precedence))
          } else {
            return Err(
              response!(
                Wrong("reached EOF in operation"),
                self.source.file,
                position
              )
            )
          }
        } else {
          expression_stack.push(self.parse_atom()?);
          operator_stack.push((operator, precedence))
        }
      }

      let right = expression_stack.pop().unwrap();
      let left  = expression_stack.pop().unwrap();

      expression_stack.push(
        Expression::new(
          ExpressionNode::Binary(Rc::new(left), operator_stack.pop().unwrap().0, Rc::new(right)),
          self.current_position(),
        )
      );
    }

    let expression = expression_stack.pop().unwrap();

    Ok(
      Expression::new(
        expression.node,
        self.span_from(left_position)
      )
    )
  }

  fn parse_declaration(&mut self, left: Expression<'p>) -> Result<Statement<'p>, ()> {
    match self.current_lexeme().as_str() {
      ":" => {
        self.next()?;

        let position = left.pos.clone();

        match self.current_lexeme().as_str() {

          "=" => {
            self.next()?;

            let right    = Some(self.parse_expression()?);
            let position = left.pos.clone();

            Ok(
              Statement::new(
                StatementNode::Variable(
                  Type::Nil,
                  left,
                  right,
                ),

                position,
              )
            )
          },

          _ => {
            let t = self.parse_type()?;

            match self.current_lexeme().as_str() {
              "=" => {
                self.next()?;

                let right    = Some(self.parse_expression()?);
                let position = left.pos.clone();

                Ok(
                  Statement::new(
                    StatementNode::Variable(
                      t,
                      left,
                      right,
                    ),

                    position,
                  )
                )
              },

              _ => Ok(
                Statement::new(
                  StatementNode::Variable(
                    t,
                    left,
                    None,
                  ),

                  position,
                )
              )
            }
          }
        }
      },

      _ => Err(
        response!(
          Wrong("invalid declaration without `:`"),
          self.source.file,
          self.current_position()
        )
      )
    }
  }



  fn parse_type(&mut self) -> Result<Type, ()> {
    use self::TokenType::*;

    let t = match *self.current_type() {
      Identifier => match self.eat()?.as_str() {
        "str"   => Type::String,
        "int"   => Type::Int,
        "float" => Type::Float,
        "bool"  => Type::Bool,
        "char"  => Type::Char,
        id      => Type::Id(id.to_owned()),
      },

      _ => return Err(
        response!(
          Wrong(format!("expected type found `{}`", self.current_lexeme())),
          self.source.file,
          self.current_position()
        )
      )
    };

    Ok(t)
  }



  fn next(&mut self) -> Result<(), ()> {
    if self.index <= self.tokens.len() {
      self.index += 1;
      Ok(())
    } else {
      Err(
        response!(
          Wrong("moving outside token stack"),
          self.source.file
        )
      )
    }
  }

  fn remaining(&self) -> usize {
    self.tokens.len().saturating_sub(self.index)
  }

  fn current_position(&self) -> TokenElement<'p> {
    let current = self.current();

    TokenElement::Pos(
      current.line,
      current.slice
    )
  }

  fn span_from(&self, left_position: TokenElement<'p>) -> TokenElement<'p> {
    match left_position {
      TokenElement::Pos(ref line, ref slice) => if let TokenElement::Pos(_, ref slice2) = self.current_position() {
        TokenElement::Pos(*line, (slice.0, if slice2.1 < line.1.len() { slice2.1 } else { line.1.len() } ))
      } else {
        left_position.clone()
      },

      _ => left_position.clone(),
    }
  }

  fn current(&self) -> &'p Token<'p> {
    if self.index > self.tokens.len() - 1 {
      &self.tokens[self.tokens.len() - 1]
    } else {
      &self.tokens[self.index]
    }
  }

  fn eat(&mut self) -> Result<String, ()> {
    let lexeme = self.current().lexeme.clone();
    self.next()?;

    Ok(lexeme)
  }

  fn eat_lexeme(&mut self, lexeme: &str) -> Result<String, ()> {
    if self.current_lexeme() == lexeme {
      let lexeme = self.current().lexeme.clone();
      self.next()?;

      Ok(lexeme)
    } else {
      Err(
        response!(
          Wrong(format!("expected `{}`, found `{}`", lexeme, self.current_lexeme())),
          self.source.file,
          self.current_position()
        )
      )
    }
  }

  fn eat_type(&mut self, token_type: &TokenType) -> Result<String, ()> {
    if self.current_type() == token_type {
      let lexeme = self.current().lexeme.clone();
      self.next()?;

      Ok(lexeme)
    } else {
      Err(
        response!(
          Wrong(format!("expected `{}`, found `{}`", token_type, self.current_type())),
          self.source.file,
          self.current_position()
        )
      )
    }
  }

  fn current_lexeme(&self) -> String {
    self.current().lexeme.clone()
  }

  fn current_type(&self) -> &TokenType {
    &self.current().token_type
  }

  fn expect_type(&self, token_type: TokenType) -> Result<(), ()> {
    if self.current_type() == &token_type {
      Ok(())
    } else {
      Err(
        response!(
          Wrong(format!("expected `{}`, found `{}`", token_type, self.current_type())),
          self.source.file
        )
      )
    }
  }
}