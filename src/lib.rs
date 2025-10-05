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
    stack_node: Vec<Node>,
    stack_operator: Vec<Operator>,
}

impl Parser {
    pub fn new<'a>(operators: impl IntoIterator<Item = &'a str>) -> Self {
        let mut priorities: HashMap<String, usize> = HashMap::new();
        for (index, operator) in operators.into_iter().enumerate() {
            priorities.insert(operator.to_string(), index);
        }
        Parser {
            priorities: priorities,
            stack_node: Vec::new(),
            stack_operator: Vec::new(),
        }
    }

    fn insert_placeholder(&mut self, token_before: Option<&str>, token_after: Option<&str>) {
        let needs_placeholder_after = match token_before {
            None => true,
            Some("(") => true,
            Some(token) if self.priorities.get(token).is_some() => true,
            _ => false,
        };
        let needs_placeholder_before = match token_after {
            None => true,
            Some(")") => true,
            Some(token) if self.priorities.get(token).is_some() => true,
            _ => false,
        };
        if needs_placeholder_after && needs_placeholder_before {
            self.stack_node.push(Node::Placeholder())
        }
    }

    fn pop_operator(&mut self) -> Option<()> {
        match self.stack_operator.pop()? {
            Operator::Operator(label, _) => {
                let o2 = self.stack_node.pop()?;
                let o1 = self.stack_node.pop()?;
                let node = Node::Operator(label, Box::new(o1), Box::new(o2));
                Some(self.stack_node.push(node))
            }
            _ => None,
        }
    }

    fn pop_operators_ge(&mut self, priority0: usize) -> Option<()> {
        while let Some(&Operator::Operator(_, priority)) = self.stack_operator.last()
            && priority >= priority0
        {
            self.pop_operator()?;
        }
        Some(())
    }

    pub fn parse<'a>(&mut self, input: impl IntoIterator<Item = &'a str>) -> Option<Node> {
        let mut token_previous: Option<&str> = None;
        for token in input {
            self.insert_placeholder(token_previous, Some(token));
            match token {
                "(" => {
                    self.stack_operator.push(Operator::Parenthesis());
                }
                ")" => {
                    self.pop_operators_ge(0)?;
                    let _ = self.stack_operator.pop()?;
                    let node = self.stack_node.pop()?;
                    self.stack_node.push(Node::Parentheses(Box::new(node)));
                }
                _ => match self.priorities.get(token) {
                    Some(&priority) => {
                        self.pop_operators_ge(priority)?;
                        let operator = Operator::Operator(token.to_string(), priority);
                        self.stack_operator.push(operator);
                    }
                    _ => {
                        self.stack_node.push(Node::Primitive(token.to_string()));
                    }
                },
            }
            token_previous = Some(token);
        }
        self.insert_placeholder(token_previous, None);
        self.pop_operators_ge(0)?;
        match self.stack_operator.is_empty() {
            true => self.stack_node.pop(),
            false => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_1() {
        let mut parser = Parser::new(["+", "*"]);
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
        let mut parser = Parser::new(["+", "-", "*"]);
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
        let mut parser = Parser::new(["+", "?", "*"]);
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
        let mut parser = Parser::new(["+", "*"]);
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
        let mut parser = Parser::new(["+", "-", "*"]);
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
        let mut parser = Parser::new(["+", "?", "*"]);
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
        let mut parser = Parser::new(["*"]);
        assert_eq!(parser.parse(["p", "q", "*", "r", "s"]), None);
    }
}
