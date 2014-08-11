#![crate_name = "small_step_simple"]
#![crate_type = "lib"]

#![feature(macro_rules)]

extern crate std;

use std::fmt::Show;
use std::fmt::Formatter;
use std::fmt::Result;
use std::collections::hashmap::HashMap;


#[deriving(Clone,PartialEq)]
pub enum Element {
    Number(i64),
    Add(Box<Element>, Box<Element>),
    Multiply(Box<Element>, Box<Element>),
    Boolean(bool),
    LessThan(Box<Element>, Box<Element>),
    Variable(String),
    Assign(String, Box<Element>),
    Sequence(Box<Element>, Box<Element>),
    DoNothing
}

impl Show for Element {
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
            DoNothing => write!(f, "do-nothing")
        }
    }
}

impl Element {
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
        }
    }

    pub fn value(&self) -> i64 {
        match *self {
            Number(val) => val,
            _ => fail!("type mismatch in value")
        }
    }

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
            DoNothing => { DoNothing }
            _ => fail!("type mismatch in reduce: {}", *self)
        }
    }
}


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

pub struct Machine {
    expression: Box<Element>,
    environment: HashMap<String, Box<Element>>
}

impl Machine {
    pub fn new(expression: Box<Element>, map: HashMap<String, Box<Element>>) -> Machine {
        Machine {
            expression: expression,
            environment: map
        }
    }

    pub fn new_with_empty_env(expression: Box<Element>) -> Machine {
        let map: HashMap<String, Box<Element>> = HashMap::new();
        Machine {
            expression: expression,
            environment: map
        }
    }

    pub fn step(&mut self) {
        self.expression = box self.expression.reduce(&mut self.environment)
    }

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

    let mut stderr = io::stderr();

    assert_eq!(true, sequence.is_reducible());
    let sequence = sequence.reduce(&mut env);
    assert_eq!(true, sequence.is_reducible());
    let sequence = sequence.reduce(&mut env);
    assert_eq!(false, sequence.is_reducible());
}
