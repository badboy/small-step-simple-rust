//! This is an implementation of the small-step approach to the SIMPLE language as introduced by
//! [Tom Stuart](https://twitter.com/tomstuart) in "Understanding Computation", Chapter 1, "The Meaning of Programs".
//! See his website: <http://computationbook.com/>.
//!
//! The usage is pretty simple. As there is no parser for SIMPLE (yet?) you have to write the AST
//! yourself. A few macros are provided for easy access. You can then create a virtual machine and
//! pass this AST plus an environment hash. When calling `run`, the machine steps through the code,
//! reducing it until it reaches a point where no further reduction is possible.
//!
//! ```ignore
//! let mut env = HashMap::new();
//! env.insert("y".to_string(), number!(1));
//!
//! let mut m = Machine::new(
//!     sequence!(
//!         assign!("x", number!(3)),
//!         assign!("res", add!(add!(number!(38), variable!("x")), variable!("y")))
//!         ),
//!         env
//!     );
//!
//! m.run();
//! // At this point `res` in the HashMap will be `Number(42)`
//! ```
//!
//!
//! The code is much larger as the equivalent Ruby code. This is both due to the restricitions
//! of Rust (explicit types and everything, a good thing) and my non-existing experience with Rust
//! at all (this is my first Rust code larger than a simple "Hello World")

#![feature(box_syntax,box_patterns)]

use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Result;
use std::collections::hash_map::HashMap;

/// Our AST elements.
#[derive(Clone,PartialEq)]
pub enum Element {
    /// A simple number object, this cannot be reduced further.
    Number(i64),
    /// An addition of two elements.
    Add(Box<Element>, Box<Element>),
    /// A multiplication of two elements.
    Multiply(Box<Element>, Box<Element>),
    /// A simple boolean object, this cannot be reduced further.
    Boolean(bool),
    /// A less-than relation check of two elements. Elements should reduce to a number to be
    /// comparable.
    LessThan(Box<Element>, Box<Element>),
    /// A variable, will be replaced by its value when reducing.
    Variable(String),
    /// A variable assignment. Only completely reduced values are assigned. No type checks.
    Assign(String, Box<Element>),
    /// A sequence of two elements. The first element is reduced completely before the second is
    /// touched.
    Sequence(Box<Element>, Box<Element>),
    /// A if-else block. Condition needs to reduce to a Boolean. No type checking.
    /// If `condition` reduces to true, the `consequence` is used furhter, otherwise the `alternative`
    IfElse(Box<Element>, Box<Element>, Box<Element>),
    /// A while loop. Runs until the `condition` reduces to false.
    While(Box<Element>, Box<Element>),
    /// A simple no-op statement.
    DoNothing
}

/// Macros to create boxed AST elements.
macro_rules! number(
    ($val:expr) => (
        box Element::Number($val)
    );
);
macro_rules! add(
    ($l:expr, $r:expr) => (
        box Element::Add($l, $r)
    )
);
macro_rules! multiply(
    ($l:expr, $r:expr) => (
        box Element::Multiply($l, $r)
    )
);
macro_rules! boolean(
    ($val:expr) => (
        box Element::Boolean($val)
    )
);
macro_rules! less_than(
    ($l:expr, $r:expr) => (
        box Element::LessThan($l, $r)
    )
);
macro_rules! variable(
    ($v:expr) => (
        box Element::Variable($v.to_string())
    )
);
macro_rules! assign(
    ($name:expr, $exp:expr) => (
        box Element::Assign($name.to_string(), $exp)
    )
);
macro_rules! sequence(
    ($first:expr, $second:expr) => (
        box Element::Sequence($first, $second)
    )
);
macro_rules! ifelse(
    ($condition:expr, $consequence:expr, $alternative:expr) => (
        box Element::IfElse($condition, $consequence, $alternative)
    )
);
macro_rules! if_(
    ($condition:expr, $consequence:expr) => (
        box Element::IfElse($condition, $consequence, box Element::DoNothing)
    )
);
macro_rules! while_(
    ($condition:expr, $body:expr) => (
        box Element::While($condition, $body)
    )
);


impl Debug for Element {
    /// Output a user-readable representation of the expression
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            Element::Number(ref value) => write!(f, "{:?}", value),
            Element::Add(ref l, ref r) => write!(f, "{:?} + {:?}", l, r),
            Element::Multiply(ref l, ref r) => write!(f, "{:?} * {:?}", l, r),
            Element::LessThan(ref l, ref r) => write!(f, "{:?} < {:?}", l, r),
            Element::Boolean(ref b) => write!(f, "{:?}", b),
            Element::Variable(ref value) => write!(f, "{}", value),
            Element::Assign(ref name, ref val) => write!(f, "{} = {:?}", name, val),
            Element::Sequence(ref first, ref second) => write!(f, "{:?}; {:?}", first, second),
            Element::IfElse(ref cond, ref cons, ref alt) => {
                write!(f, "if ({:?}) [ {:?} ] else [ {:?} ]", cond, cons, alt)
            }
            Element::While(ref cond, ref body) => {
                write!(f, "while ({:?}) [ {:?} ]", cond, body)
            }
            Element::DoNothing => write!(f, "do-nothing")
        }
    }
}

impl Element {
    /// Wether or not an expression is reducible. See Element for more info.
    pub fn is_reducible(&self) -> bool {
        match *self {
            Element::Number(_) => false,
            Element::Boolean(_) => false,
            Element::DoNothing => false,
            Element::Add(_, _) => true,
            Element::Multiply(_, _) => true,
            Element::LessThan(_, _) => true,
            Element::Variable(_) => true,
            Element::Assign(_, _) => true,
            Element::Sequence(_, _) => true,
            Element::IfElse(_, _, _) => true,
            Element::While(_, _) => true,
        }
    }

    /// Get the actual value of a Number.
    /// Fails for other elements than Number and Boolean.
    /// Boolean maps to Integers: true=1, false=0.
    pub fn value(&self) -> i64 {
        match *self {
            Element::Number(val) => val,
            Element::Boolean(true) => 1,
            Element::Boolean(false) => 0,
            _ => panic!("type mismatch in value")
        }
    }

    /// Reduce the expression according to the rules for the current element.
    pub fn reduce(&self, environment: &mut HashMap<String, Box<Element>>) -> Element {
        match *self {
            Element::Add(ref l, ref r) => {
                if l.is_reducible() {
                    Element::Add(box l.reduce(environment), r.clone())
                } else if r.is_reducible() {
                    Element::Add(l.clone(), box r.reduce(environment))
                } else {
                    Element::Number(l.value() + r.value())
                }
            },
            Element::Multiply(ref l, ref r) => {
                if l.is_reducible() {
                    Element::Multiply(box l.reduce(environment), r.clone())
                } else if r.is_reducible() {
                    Element::Multiply(l.clone(), box r.reduce(environment))
                } else {
                    Element::Number(l.value() * r.value())
                }
            },
            Element::LessThan(ref l, ref r) => {
                if l.is_reducible() {
                    Element::LessThan(box l.reduce(environment), r.clone())
                } else if r.is_reducible() {
                    Element::LessThan(l.clone(), box r.reduce(environment))
                } else {
                    Element::Boolean(l.value() < r.value())
                }
            },
            Element::Variable(ref v) => {
                match environment.get(v) {
                    Some(v) => {
                        *v.clone()
                    },
                    None => Element::DoNothing
                }
            },
            Element::Assign(ref name, ref expression) => {
                if expression.is_reducible() {
                    Element::Assign(name.clone(), box expression.reduce(environment))
                } else {
                    environment.insert(name.clone(), expression.clone());
                    Element::DoNothing
                }
            },
            Element::Sequence(box Element::DoNothing, ref second) => {
                *second.clone()
            },
            Element::Sequence(ref first, ref second) => {
                Element::Sequence(box first.reduce(environment), second.clone())
            },
            Element::IfElse(box Element::Boolean(true), ref cons, _) => {
                *cons.clone()
            },
            Element::IfElse(box Element::Boolean(false), _, ref alt) => {
                *alt.clone()
            },
            Element::IfElse(ref cond, ref cons, ref alt) => {
                if cond.is_reducible() {
                    Element::IfElse(box cond.reduce(environment), cons.clone(), alt.clone())
                } else {
                    panic!("Condition in if not reducible (but not bool): {:?}", cond)
                }
            },
            Element::While(ref cond, ref body) => {
                Element::IfElse(cond.clone(), box Element::Sequence(body.clone(), box self.clone()), box Element::DoNothing)
            }
            Element::DoNothing => { Element::DoNothing }
            _ => panic!("type mismatch in reduce: {:?}", *self)
        }
    }
}

#[test]
fn test_types_are_creatable() {
    let i = number!(3);
    assert_eq!("3".to_string(), format!("{:?}", i));
    assert_eq!(false, i.is_reducible());

    let i = add!(number!(3), number!(4));
    assert_eq!("3 + 4".to_string(), format!("{:?}", i));
    assert_eq!(true, i.is_reducible());

    let i = multiply!(
        add!(number!(3), number!(4)),
        number!(2));
    assert_eq!("3 + 4 * 2".to_string(), format!("{:?}", i));
    assert_eq!(true, i.is_reducible());

    let i = boolean!(true);
    assert_eq!("true".to_string(), format!("{:?}", i));
    assert_eq!(false, i.is_reducible());

    let i = less_than!(number!(2), number!(3));
    assert_eq!("2 < 3".to_string(), format!("{:?}", i));
    assert_eq!(true, i.is_reducible());
}

#[test]
fn test_expression_is_reducible() {
    let expression = add!(
        multiply!(number!(1), number!(2)),
        multiply!(number!(3), number!(4))
    );

    assert_eq!(true, expression.is_reducible())
}

#[test]
fn test_expression_reduces() {
    let expression = add!(
        multiply!(number!(1), number!(2)),
        multiply!(number!(3), number!(4))
    );

    let mut empty_env = HashMap::new();
    assert_eq!("1 * 2 + 3 * 4".to_string(), format!("{:?}", expression));
    let red = expression.reduce(&mut empty_env);
    assert_eq!("2 + 3 * 4".to_string(), format!("{:?}", red));
    let red = red.reduce(&mut empty_env);
    assert_eq!("2 + 12".to_string(), format!("{:?}", red));
    let red = red.reduce(&mut empty_env);
    assert_eq!("14".to_string(), format!("{:?}", red));
    assert_eq!(false, red.is_reducible())
}

/// Our virtual machine, executing our constructed AST step-by-step
pub struct Machine {
    expression: Box<Element>,
    environment: HashMap<String, Box<Element>>
}

impl Machine {
    /// Create a new machine with a given expression and an environment
    pub fn new(expression: Box<Element>, map: HashMap<String, Box<Element>>) -> Machine {
        Machine {
            expression: expression,
            environment: map
        }
    }

    /// Create a new machine with a given expression and an _empty_ environment
    pub fn new_with_empty_env(expression: Box<Element>) -> Machine {
        let map: HashMap<String, Box<Element>> = HashMap::new();
        Machine {
            expression: expression,
            environment: map
        }
    }

    /// As the environment is passed in immutable, we need to clone it to get it back
    pub fn clone_env(&self) -> HashMap<String, Box<Element>> {
        self.environment.clone()
    }

    /// Reduce one step of our current expression
    pub fn step(&mut self) {
        self.expression = box self.expression.reduce(&mut self.environment)
    }

    /// Reduce until we reached a non-reducible expression.
    /// This prints the current expression before each step.
    pub fn run(&mut self) {
        while self.expression.is_reducible() {
            println!("{:?}", self.expression);
            self.step()
        }

        println!("{:?}", self.expression);
    }
}

#[test]
fn test_machine_reduces_algebraic_expression() {
    println!("Starting the machine…");
    let mut m = Machine::new_with_empty_env(
        multiply!(
            add!(number!(3), number!(4)),
            number!(2)
            )
        );

    m.run();

    println!("All done!");
}

#[test]
fn test_reduces_boolean_expression() {
    let i = less_than!(number!(2), number!(3));
    assert_eq!("2 < 3".to_string(), format!("{:?}", i));
    assert_eq!(true, i.is_reducible());

    let mut empty_env = HashMap::new();
    let i = box i.reduce(&mut empty_env);
    assert_eq!("true".to_string(), format!("{:?}", i));
    assert_eq!(false, i.is_reducible());
}

#[test]
fn test_instantiate_variable_expression() {
    let v = variable!("x".to_string());
    assert_eq!("x".to_string(), format!("{:?}", v));
    assert_eq!(true, v.is_reducible());

    let mut empty_env = HashMap::new();
    empty_env.insert("x".to_string(), number!(1));
    let v = v.reduce(&mut empty_env);
    assert_eq!("1".to_string(), format!("{:?}", v));
}

#[test]
fn test_machine_reduces_with_variables() {
    let mut env = HashMap::new();
    env.insert("x".to_string(), number!(3));
    env.insert("y".to_string(), number!(4));

    let mut m = Machine::new(
        add!(variable!("x"), variable!("y")),
        env);


    m.run();

    let mut env = HashMap::new();
    env.insert("x".to_string(), number!(3));
    env.insert("y".to_string(), number!(4));

    let exp = add!(variable!("x"), variable!("y"));
    assert_eq!("x + y".to_string(), format!("{:?}", exp));
    let exp = exp.reduce(&mut env);
    assert_eq!("3 + y".to_string(), format!("{:?}", exp));

    let exp = exp.reduce(&mut env);
    assert_eq!("3 + 4".to_string(), format!("{:?}", exp));

    let exp = exp.reduce(&mut env);
    assert_eq!("7".to_string(), format!("{:?}", exp));
}

#[test]
fn test_assignment_initializer() {
    let assignment = assign!("x", number!(1));

    assert_eq!("x = 1".to_string(), format!("{:?}", assignment));
    assert_eq!(true, assignment.is_reducible());
}

#[test]
fn test_assigment_is_reduced() {
    let assignment = assign!("x", number!(1));

    let mut env = HashMap::new();
    let assignment = assignment.reduce(&mut env);

    let ref val = env["x".to_string()];
    assert_eq!(Element::DoNothing, assignment);
    assert_eq!(1, (*val).value());
}

#[test]
fn test_sequence_is_reduced() {
    let sequence = sequence!(
        box Element::DoNothing,
        add!(number!(1), number!(2))
        );

    // do-nothing; 1 + 1
    // 1 + 1
    // 3

    let mut env = HashMap::new();

    assert_eq!(true, sequence.is_reducible());
    let sequence = sequence.reduce(&mut env);
    assert_eq!(true, sequence.is_reducible());
    let sequence = sequence.reduce(&mut env);
    assert_eq!(false, sequence.is_reducible());
}

#[test]
fn test_expression_with_assignment_and_variables_runs() {
    let mut env = HashMap::new();
    env.insert("y".to_string(), number!(1));

    let mut m = Machine::new(
        sequence!(
            assign!("x", number!(3)),
            assign!("res", add!(add!(number!(38), variable!("x")), variable!("y")))
            ),
            env
        );

    m.run();

    let env = m.clone_env();

    assert_eq!(1, env.get(&"y".to_string()).unwrap().value());
    assert_eq!(3, env.get(&"x".to_string()).unwrap().value());
    assert_eq!(42, env.get(&"res".to_string()).unwrap().value());
}

#[test]
fn test_if_is_reduced_true () {
    let mut env = HashMap::new();
    let if_block = ifelse!(
        boolean!(true),
        number!(1),
        number!(2)
        );
    // if(true) [ 1 ] else [ 2 ]
    // 1


    assert_eq!(true, if_block.is_reducible());
    assert_eq!("if (true) [ 1 ] else [ 2 ]".to_string(), format!("{:?}", if_block));

    let if_block = if_block.reduce(&mut env);
    assert_eq!("1".to_string(), format!("{:?}", if_block));
}

#[test]
fn test_if_is_reduced_false () {
    let mut env = HashMap::new();
    let if_block = ifelse!(
        boolean!(false),
        number!(1),
        number!(2)
        );
    // if(true) [ 1 ] else [ 2 ]
    // 2


    assert_eq!(true, if_block.is_reducible());
    assert_eq!("if (false) [ 1 ] else [ 2 ]".to_string(), format!("{:?}", if_block));

    let if_block = if_block.reduce(&mut env);
    assert_eq!("2".to_string(), format!("{:?}", if_block));
}

#[test]
fn test_if_is_reduced_with_expression () {
    let mut env = HashMap::new();
    let if_block = ifelse!(
        less_than!(number!(1), number!(2)),
        number!(1),
        number!(2)
        );
    // if(1 < 2) [ 1 ] else [ 2 ]
    // if(true) [ 1 ] else [ 2 ]
    // 1


    assert_eq!(true, if_block.is_reducible());
    assert_eq!("if (1 < 2) [ 1 ] else [ 2 ]".to_string(), format!("{:?}", if_block));

    let if_block = if_block.reduce(&mut env);
    assert_eq!("if (true) [ 1 ] else [ 2 ]".to_string(), format!("{:?}", if_block));

    let if_block = if_block.reduce(&mut env);
    assert_eq!("1".to_string(), format!("{:?}", if_block));
}

#[test]
fn test_if_without_else () {
    let mut env = HashMap::new();
    let if_block = if_!(boolean!(true), number!(1));
    // if(true) [ 1 ] else [ do-nothing ]
    // 1


    assert_eq!(true, if_block.is_reducible());
    assert_eq!("if (true) [ 1 ] else [ do-nothing ]".to_string(), format!("{:?}", if_block));

    let if_block = if_block.reduce(&mut env);
    assert_eq!("1".to_string(), format!("{:?}", if_block));
}

#[test]
fn test_expression_with_assignment_and_if() {
    let env = HashMap::new();

    let mut m = Machine::new(
        sequence!(
            assign!("x", boolean!(false)),
            ifelse!(
                variable!("x"),
                assign!("y", number!(1)),
                assign!("y", number!(42))
            )
            ),
            env
        );

    m.run();

    let env = m.clone_env();

    assert_eq!(0, env.get(&"x".to_string()).unwrap().value());
    assert_eq!(42, env.get(&"y".to_string()).unwrap().value());
}

#[test]
fn test_while_loops () {
    let mut env = HashMap::new();
    env.insert("x".to_string(), number!(1));

    let while_loop = while_!(
        less_than!(variable!("x"), number!(2)),
        assign!("x", add!(variable!("x"), number!(1)))
        );
    //  1. while (x < 2) [ x = x + 1 ]
    //  2. if (x < 2) [ x = x +1 ; while (x < 2) [ x = x + 1 ] ] else [ do-nothing ];
    //  3. if (1 < 2) [ x = x +1 ; while (x < 2) [ x = x + 1 ] ] else [ do-nothing ];
    //  4. if (true) [ x = x +1 ; while (x < 2) [ x = x + 1 ] ] else [ do-nothing ];
    //  5. x = x + 1; while (x < 2) [ x = x + 1 ]
    //  6. x = 1 + 1; while (x < 2) [ x = x + 1 ]
    //  7. x = 2; while (x < 2) [ x = x + 1 ]
    //  8. do-nothing; while (x < 2) [ x = x + 1 ]
    //  9. while (x < 2) [ x = x + 1 ]
    // 10. if (x < 2) [ x = x +1 ; while (x < 2) [ x = x + 1 ] ] else [ do-nothing ];
    // 11. if (2 < 2) [ x = x +1 ; while (x < 2) [ x = x + 1 ] ] else [ do-nothing ];
    // 12. if (false) [ x = x +1 ; while (x < 2) [ x = x + 1 ] ] else [ do-nothing ];
    // 13. do-nothing


    assert_eq!(true, while_loop.is_reducible());
    assert_eq!("while (x < 2) [ x = x + 1 ]".to_string(), format!("{:?}", while_loop));

    let while_loop = while_loop.reduce(&mut env);
    assert_eq!(
        "if (x < 2) [ x = x + 1; while (x < 2) [ x = x + 1 ] ] else [ do-nothing ]".to_string(),
        format!("{:?}", while_loop));
}

#[test]
fn test_while_loops_fully_with_machine () {
    let mut env = HashMap::new();
    env.insert("x".to_string(), number!(1));

    //  1. while (x < 5) [ x = x * 3 ]
    let mut m = Machine::new(
        while_!(
            less_than!(variable!("x"), number!(5)),
            assign!("x", multiply!(variable!("x"), number!(3)))
            ),
            env
            );

    m.run();

    let env = m.clone_env();

    assert_eq!(9, env.get(&"x".to_string()).unwrap().value());
}
