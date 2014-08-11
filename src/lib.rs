#![crate_name = "small_step_simple"]
#![crate_type = "lib"]

//! This is an implementation of the small-step approach to the SIMPLE language as introduced by
//! [Tom Stuart](https://twitter.com/tomstuart) in "Understanding Computation", Chapter 1, "The Meaning of Programs".
//! See his website: <http://computationbook.com/>.
//!
//! The usage is pretty simple. As there is no parser for SIMPLE (yet?) you have to write the AST
//! yourself. A few macros are provided for easy access. You can then create a virtual machine and
//! pass this AST plus an environment hash. When calling `run`, the machine steps through the code,
//! reducing it until it reaches a point where no further reduction is possible.
//!
//! ```
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
//! // Add this point `res` in the HashMap will be `Number(42)`
//! ```
//!
//!
//! The code is much larger as the equivalent Ruby code. This is both due to the restricitions
//! of Rust (explicit types and everything, a good thing) and my non-existing experience with Rust
//! at all (this is my first Rust code larger than a simple "Hello World")

#![feature(macro_rules)]

extern crate std;

use std::fmt::Show;
use std::fmt::Formatter;
use std::fmt::Result;
use std::collections::hashmap::HashMap;

/// Our AST elements.
#[deriving(Clone,PartialEq)]
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
    /// A simple no-op statement.
    DoNothing
}

/// Macros to create boxed AST elements.
macro_rules! number(
    ($val:expr) => (
        box Number($val)
    );
)
macro_rules! add(
    ($l:expr, $r:expr) => (
        box Add($l, $r)
    );
)
macro_rules! multiply(
    ($l:expr, $r:expr) => (
        box Multiply($l, $r)
    );
)
macro_rules! boolean(
    ($val:expr) => (
        box Boolean($val)
    );
)
macro_rules! less_than(
    ($l:expr, $r:expr) => (
        box LessThan($l, $r)
    );
)
macro_rules! variable(
    ($v:expr) => (
        box Variable($v.to_string())
    );
)
macro_rules! assign(
    ($name:expr, $exp:expr) => (
        box Assign($name.to_string(), $exp)
    );
)
macro_rules! sequence(
    ($first:expr, $second:expr) => (
        box Sequence($first, $second)
    );
)
macro_rules! ifelse(
    ($condition:expr, $consequence:expr, $alternative:expr) => (
        box IfElse($condition, $consequence, $alternative)
    );
)
macro_rules! if_(
    ($condition:expr, $consequence:expr) => (
        box IfElse($condition, $consequence, box DoNothing)
    );
)


impl Show for Element {
    /// Output a user-readable representation of the expression
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            Number(ref value) => write!(f, "{}", value),
            Add(ref l, ref r) => write!(f, "{} + {}", l, r),
            Multiply(ref l, ref r) => write!(f, "{} * {}", l, r),
            LessThan(ref l, ref r) => write!(f, "{} < {}", l, r),
            Boolean(ref b) => write!(f, "{}", b),
            Variable(ref value) => write!(f, "{}", value),
            Assign(ref name, ref val) => write!(f, "{} = {}", name, val),
            Sequence(ref first, ref second) => write!(f, "{}; {}", first, second),
            IfElse(ref cond, ref cons, ref alt) => {
                write!(f, "if ({}) [ {} ] else [ {} ]", cond, cons, alt)
            }
            DoNothing => write!(f, "do-nothing")
        }
    }
}

impl Element {
    /// Wether or not an expression is reducible. See Element for more info.
    pub fn is_reducible(&self) -> bool {
        match *self {
            Number(_) => false,
            Boolean(_) => false,
            DoNothing => false,
            Add(_, _) => true,
            Multiply(_, _) => true,
            LessThan(_, _) => true,
            Variable(_) => true,
            Assign(_, _) => true,
            Sequence(_, _) => true,
            IfElse(_, _, _) => true,
        }
    }

    /// Get the actual value of a Number. Fails for other elements than Number.
    pub fn value(&self) -> i64 {
        match *self {
            Number(val) => val,
            _ => fail!("type mismatch in value")
        }
    }

    /// Reduce the expression according to the rules for the current element.
    pub fn reduce(&self, environment: &mut HashMap<String, Box<Element>>) -> Element {
        match *self {
            Add(ref l, ref r) => {
                if l.is_reducible() {
                    Add(box l.reduce(environment), r.clone())
                } else if r.is_reducible() {
                    Add(l.clone(), box r.reduce(environment))
                } else {
                    Number(l.value() + r.value())
                }
            },
            Multiply(ref l, ref r) => {
                if l.is_reducible() {
                    Multiply(box l.reduce(environment), r.clone())
                } else if r.is_reducible() {
                    Multiply(l.clone(), box r.reduce(environment))
                } else {
                    Number(l.value() * r.value())
                }
            },
            LessThan(ref l, ref r) => {
                if l.is_reducible() {
                    LessThan(box l.reduce(environment), r.clone())
                } else if r.is_reducible() {
                    LessThan(l.clone(), box r.reduce(environment))
                } else {
                    Boolean(l.value() < r.value())
                }
            },
            Variable(ref v) => {
                match environment.find(v) {
                    Some(v) => {
                        *v.clone()
                    },
                    None => DoNothing
                }
            },
            Assign(ref name, ref expression) => {
                if expression.is_reducible() {
                    Assign(name.clone(), box expression.reduce(environment))
                } else {
                    environment.insert(name.clone(), expression.clone());
                    DoNothing
                }
            },
            Sequence(box DoNothing, ref second) => {
                *second.clone()
            },
            Sequence(ref first, ref second) => {
                Sequence(box first.reduce(environment), second.clone())
            },
            IfElse(box Boolean(true), ref cons, _) => {
                *cons.clone()
            },
            IfElse(box Boolean(false), _, ref alt) => {
                *alt.clone()
            },
            IfElse(ref cond, ref cons, ref alt) => {
                if cond.is_reducible() {
                    IfElse(box cond.reduce(environment), cons.clone(), alt.clone())
                } else {
                    fail!("Condition in if not reducible (but not bool): {}", cond)
                }
            },
            DoNothing => { DoNothing }
            _ => fail!("type mismatch in reduce: {}", *self)
        }
    }
}

#[test]
fn test_types_are_creatable() {
    let i = number!(3);
    assert_eq!("3".to_string(), format!("{}", i));
    assert_eq!(false, i.is_reducible());

    let i = add!(number!(3), number!(4));
    assert_eq!("3 + 4".to_string(), format!("{}", i));
    assert_eq!(true, i.is_reducible());

    let i = multiply!(
        add!(number!(3), number!(4)),
        number!(2));
    assert_eq!("3 + 4 * 2".to_string(), format!("{}", i));
    assert_eq!(true, i.is_reducible());

    let i = boolean!(true);
    assert_eq!("true".to_string(), format!("{}", i));
    assert_eq!(false, i.is_reducible());

    let i = less_than!(number!(2), number!(3));
    assert_eq!("2 < 3".to_string(), format!("{}", i));
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
    assert_eq!("1 * 2 + 3 * 4".to_string(), format!("{}", expression));
    let red = expression.reduce(&mut empty_env);
    assert_eq!("2 + 3 * 4".to_string(), format!("{}", red));
    let red = red.reduce(&mut empty_env);
    assert_eq!("2 + 12".to_string(), format!("{}", red));
    let red = red.reduce(&mut empty_env);
    assert_eq!("14".to_string(), format!("{}", red));
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
            println!("{}", self.expression);
            self.step()
        }

        println!("{}", self.expression);
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
    assert_eq!("2 < 3".to_string(), format!("{}", i));
    assert_eq!(true, i.is_reducible());

    let mut empty_env = HashMap::new();
    let i = box i.reduce(&mut empty_env);
    assert_eq!("true".to_string(), format!("{}", i));
    assert_eq!(false, i.is_reducible());
}

#[test]
fn test_instantiate_variable_expression() {
    let v = variable!("x".to_string());
    assert_eq!("x".to_string(), format!("{}", v));
    assert_eq!(true, v.is_reducible());

    let mut empty_env = HashMap::new();
    empty_env.insert("x".to_string(), number!(1));
    let v = v.reduce(&mut empty_env);
    assert_eq!("1".to_string(), format!("{}", v));
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
    assert_eq!("x + y".to_string(), format!("{}", exp));
    let exp = exp.reduce(&mut env);
    assert_eq!("3 + y".to_string(), format!("{}", exp));

    let exp = exp.reduce(&mut env);
    assert_eq!("3 + 4".to_string(), format!("{}", exp));

    let exp = exp.reduce(&mut env);
    assert_eq!("7".to_string(), format!("{}", exp));
}

#[test]
fn test_assignment_initializer() {
    let assignment = assign!("x", number!(1));

    assert_eq!("x = 1".to_string(), format!("{}", assignment));
    assert_eq!(true, assignment.is_reducible());
}

#[test]
fn test_assigment_is_reduced() {
    let assignment = assign!("x", number!(1));

    let mut env = HashMap::new();
    let assignment = assignment.reduce(&mut env);

    let val = env.get(&"x".to_string());
    assert_eq!(DoNothing, assignment);
    assert_eq!(1, (*val).value());
}

#[test]
fn test_sequence_is_reduced() {
    let sequence = sequence!(
        box DoNothing,
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

    assert_eq!(1, env.get(&"y".to_string()).value());
    assert_eq!(3, env.get(&"x".to_string()).value());
    assert_eq!(42, env.get(&"res".to_string()).value());
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
    assert_eq!("if (true) [ 1 ] else [ 2 ]".to_string(), format!("{}", if_block));

    let if_block = if_block.reduce(&mut env);
    assert_eq!("1".to_string(), format!("{}", if_block));
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
    assert_eq!("if (false) [ 1 ] else [ 2 ]".to_string(), format!("{}", if_block));

    let if_block = if_block.reduce(&mut env);
    assert_eq!("2".to_string(), format!("{}", if_block));
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
    assert_eq!("if (1 < 2) [ 1 ] else [ 2 ]".to_string(), format!("{}", if_block));

    let if_block = if_block.reduce(&mut env);
    assert_eq!("if (true) [ 1 ] else [ 2 ]".to_string(), format!("{}", if_block));

    let if_block = if_block.reduce(&mut env);
    assert_eq!("1".to_string(), format!("{}", if_block));
}

#[test]
fn test_if_without_else () {
    let mut env = HashMap::new();
    let if_block = if_!(boolean!(true), number!(1));
    // if(true) [ 1 ] else [ do-nothing ]
    // 1


    assert_eq!(true, if_block.is_reducible());
    assert_eq!("if (true) [ 1 ] else [ do-nothing ]".to_string(), format!("{}", if_block));

    let if_block = if_block.reduce(&mut env);
    assert_eq!("1".to_string(), format!("{}", if_block));
}
