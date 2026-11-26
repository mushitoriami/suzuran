# suzuran

An operator-precedence parser based on the Shunting Yard algorithm

```rust
let parser = suzuran::Parser::new(["+", "*"]);
assert_eq!(
    format!("{:?}", parser.parse(["1", "+", "2", "*", "3"]).unwrap()),
    r#"Operator("+", Primitive("1"), Operator("*", Primitive("2"), Primitive("3")))"#
);

let parser = suzuran::Parser::new(["+", "-", "*"]);
assert_eq!(
    format!("{:?}", parser.parse(["-", "(", "1", "+", "2", ")", "*", "3"]).unwrap()),
    r#"Operator("-", Placeholder, Operator("*", Parentheses(Operator("+", Primitive("1"), Primitive("2"))), Primitive("3")))"#
);
```
