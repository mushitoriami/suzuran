#![doc = include_str!("../README.md")]

use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum Node {
    Operator(String, Box<Node>, Box<Node>),
    Parentheses(Box<Node>),
    Primitive(String),
    Placeholder(),
}

enum Operator {
    Operator(String, usize),
    Parenthesis(),
}

pub struct Parser {
    priorities: HashMap<String, usize>,
}

impl Parser {
    pub fn new<'a>(operators: impl IntoIterator<Item = &'a str>) -> Self {
        let mut priorities: HashMap<String, usize> = HashMap::new();
        for (index, operator) in operators.into_iter().enumerate() {
            priorities.insert(operator.to_string(), index);
        }
        Parser { priorities }
    }

    fn insert_placeholder(
        &self,
        stack_node: &mut Vec<Node>,
        token_before: Option<&str>,
        token_after: Option<&str>,
    ) {
        let needs_placeholder_after = match token_before {
            None => true,
            Some("(") => true,
            Some(token) if self.priorities.contains_key(token) => true,
            _ => false,
        };
        let needs_placeholder_before = match token_after {
            None => true,
            Some(")") => true,
            Some(token) if self.priorities.contains_key(token) => true,
            _ => false,
        };
        if needs_placeholder_after && needs_placeholder_before {
            stack_node.push(Node::Placeholder())
        }
    }

    fn pop_operator(stack_node: &mut Vec<Node>, stack_operator: &mut Vec<Operator>) -> Option<()> {
        match stack_operator.pop()? {
            Operator::Operator(label, _) => {
                let o2 = stack_node.pop()?;
                let o1 = stack_node.pop()?;
                let node = Node::Operator(label, Box::new(o1), Box::new(o2));
                stack_node.push(node);
                Some(())
            }
            _ => None,
        }
    }

    fn pop_operators_ge(
        stack_node: &mut Vec<Node>,
        stack_operator: &mut Vec<Operator>,
        priority0: usize,
    ) -> Option<()> {
        while let Some(&Operator::Operator(_, priority)) = stack_operator.last()
            && priority >= priority0
        {
            Self::pop_operator(stack_node, stack_operator)?;
        }
        Some(())
    }

    pub fn parse<'a>(&self, input: impl IntoIterator<Item = &'a str>) -> Option<Node> {
        let mut stack_node: Vec<Node> = Vec::new();
        let mut stack_operator: Vec<Operator> = Vec::new();
        let mut token_previous: Option<&str> = None;
        for token in input {
            self.insert_placeholder(&mut stack_node, token_previous, Some(token));
            match token {
                "(" => {
                    stack_operator.push(Operator::Parenthesis());
                }
                ")" => {
                    Self::pop_operators_ge(&mut stack_node, &mut stack_operator, 0)?;
                    let _ = stack_operator.pop()?;
                    let node = stack_node.pop()?;
                    stack_node.push(Node::Parentheses(Box::new(node)));
                }
                _ => match self.priorities.get(token) {
                    Some(&priority) => {
                        Self::pop_operators_ge(&mut stack_node, &mut stack_operator, priority)?;
                        let operator = Operator::Operator(token.to_string(), priority);
                        stack_operator.push(operator);
                    }
                    _ => {
                        stack_node.push(Node::Primitive(token.to_string()));
                    }
                },
            }
            token_previous = Some(token);
        }
        self.insert_placeholder(&mut stack_node, token_previous, None);
        Self::pop_operators_ge(&mut stack_node, &mut stack_operator, 0)?;
        match stack_operator.is_empty() && stack_node.len() == 1 {
            true => stack_node.pop(),
            false => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_1() {
        let parser = Parser::new(["+", "*"]);
        assert_eq!(
            parser.parse(["1", "+", "2", "*", "3"]).unwrap(),
            Node::Operator(
                "+".to_string(),
                Box::new(Node::Primitive("1".to_string())),
                Box::new(Node::Operator(
                    "*".to_string(),
                    Box::new(Node::Primitive("2".to_string())),
                    Box::new(Node::Primitive("3".to_string()))
                ))
            )
        );
    }

    #[test]
    fn test_parse_2() {
        let parser = Parser::new(["+", "-", "*"]);
        assert_eq!(
            parser.parse(["-", "1", "*", "2", "+", "3"]).unwrap(),
            Node::Operator(
                "+".to_string(),
                Box::new(Node::Operator(
                    "-".to_string(),
                    Box::new(Node::Placeholder()),
                    Box::new(Node::Operator(
                        "*".to_string(),
                        Box::new(Node::Primitive("1".to_string())),
                        Box::new(Node::Primitive("2".to_string()))
                    ))
                )),
                Box::new(Node::Primitive("3".to_string()))
            )
        );
    }

    #[test]
    fn test_parse_3() {
        let parser = Parser::new(["+", "?", "*"]);
        assert_eq!(
            parser.parse(["1", "+", "2", "*", "3", "?"]).unwrap(),
            Node::Operator(
                "+".to_string(),
                Box::new(Node::Primitive("1".to_string())),
                Box::new(Node::Operator(
                    "?".to_string(),
                    Box::new(Node::Operator(
                        "*".to_string(),
                        Box::new(Node::Primitive("2".to_string())),
                        Box::new(Node::Primitive("3".to_string()))
                    )),
                    Box::new(Node::Placeholder())
                ))
            )
        );
    }

    #[test]
    fn test_parse_4() {
        let parser = Parser::new(["+", "*"]);
        assert_eq!(
            parser.parse(["(", "1", "+", "2", ")", "*", "3"]).unwrap(),
            Node::Operator(
                "*".to_string(),
                Box::new(Node::Parentheses(Box::new(Node::Operator(
                    "+".to_string(),
                    Box::new(Node::Primitive("1".to_string())),
                    Box::new(Node::Primitive("2".to_string()))
                )))),
                Box::new(Node::Primitive("3".to_string()))
            )
        );
    }

    #[test]
    fn test_parse_5() {
        let parser = Parser::new(["+", "-", "*"]);
        assert_eq!(
            parser
                .parse(["(", "-", "1", ")", "*", "2", "+", "3"])
                .unwrap(),
            Node::Operator(
                "+".to_string(),
                Box::new(Node::Operator(
                    "*".to_string(),
                    Box::new(Node::Parentheses(Box::new(Node::Operator(
                        "-".to_string(),
                        Box::new(Node::Placeholder()),
                        Box::new(Node::Primitive("1".to_string()))
                    )))),
                    Box::new(Node::Primitive("2".to_string()))
                )),
                Box::new(Node::Primitive("3".to_string()))
            )
        );
    }

    #[test]
    fn test_parse_6() {
        let parser = Parser::new(["+", "?", "*"]);
        assert_eq!(
            parser
                .parse(["1", "+", "2", "*", "(", "3", "?", ")"])
                .unwrap(),
            Node::Operator(
                "+".to_string(),
                Box::new(Node::Primitive("1".to_string())),
                Box::new(Node::Operator(
                    "*".to_string(),
                    Box::new(Node::Primitive("2".to_string())),
                    Box::new(Node::Parentheses(Box::new(Node::Operator(
                        "?".to_string(),
                        Box::new(Node::Primitive("3".to_string())),
                        Box::new(Node::Placeholder())
                    )))),
                ))
            )
        );
    }

    #[test]
    fn test_parse_7() {
        let parser = Parser::new(["*"]);
        assert_eq!(parser.parse(["p", "q", "*", "r", "s"]), None);
    }

    #[test]
    fn test_parse_reuse() {
        let parser = Parser::new(["+"]);
        assert_eq!(parser.parse(["(", "1", "+"]), None);
        assert_eq!(
            parser.parse(["2"]).unwrap(),
            Node::Primitive("2".to_string())
        );
    }

    #[test]
    fn test_parse_left_associative() {
        let parser = Parser::new(["-"]);
        assert_eq!(
            parser.parse(["1", "-", "2", "-", "3"]).unwrap(),
            Node::Operator(
                "-".to_string(),
                Box::new(Node::Operator(
                    "-".to_string(),
                    Box::new(Node::Primitive("1".to_string())),
                    Box::new(Node::Primitive("2".to_string()))
                )),
                Box::new(Node::Primitive("3".to_string()))
            )
        );
    }

    #[test]
    fn test_parse_unclosed_parenthesis() {
        let parser = Parser::new(["+"]);
        assert_eq!(parser.parse(["(", "1"]), None);
        assert_eq!(parser.parse(["(", "1", "+", "2"]), None);
    }

    #[test]
    fn test_parse_unopened_parenthesis() {
        let parser = Parser::new(["+"]);
        assert_eq!(parser.parse([")"]), None);
        assert_eq!(parser.parse(["1", ")"]), None);
        assert_eq!(parser.parse(["1", "+", "2", ")"]), None);
    }

    #[test]
    fn test_parse_adjacent_primitives() {
        let parser = Parser::new(["+"]);
        assert_eq!(parser.parse(["1", "(", "2", ")"]), None);
    }
}
