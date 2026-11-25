# suzuran

An operator-precedence parser based on the Shunting Yard algorithm

```
let parser = suzuran::Parser::new(["+", "*"]);
println!("{:?}", parser.parse(["1", "+", "2", "*", "3"]).unwrap());
// Operator("+", Primitive("1"), Operator("*", Primitive("2"), Primitive("3")))

let parser = suzuran::Parser::new(["+", "-", "*"]);
println!("{:?}", parser.parse(["-", "(", "1", "+", "2", ")", "*", "3"]).unwrap());
// Operator("-", Placeholder, Operator("*", Parentheses(Operator("+", Primitive("1"), Primitive("2"))), Primitive("3")))
```
