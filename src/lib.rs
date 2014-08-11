#![feature(macro_rules)]

extern crate std;

use std::fmt::Formatter;
use std::fmt::Result;

#[deriving(Clone)]
pub enum Element {
    Number(i64),
    Add(Box<Element>, Box<Element>),
    Multiply(Box<Element>, Box<Element>),
    Boolean(bool),
    LessThan(Box<Element>, Box<Element>),
    DoNothing
}

impl std::fmt::Show for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Number(ref value) => write!(f, "{}", value),
            Add(ref l, ref r) => write!(f, "{} + {}", l, r),
            Multiply(ref l, ref r) => write!(f, "{} * {}", l, r),
            LessThan(ref l, ref r) => write!(f, "{} < {}", l, r),
            Boolean(ref b) => write!(f, "{}", b),
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
        }
    }

    pub fn value(&self) -> i64 {
        match *self {
            Number(val) => val,
            _ => fail!("type mismatch in value")
        }
    }

    pub fn reduce(&self) -> Element {
        match *self {
            Add(ref l, ref r) => {
                if l.is_reducible() {
                    Add(box l.reduce(), r.clone())
                } else if r.is_reducible() {
                    Add(l.clone(), box r.reduce())
                } else {
                    Number(l.value() + r.value())
                }
            },
            Multiply(ref l, ref r) => {
                if l.is_reducible() {
                    Multiply(box l.reduce(), r.clone())
                } else if r.is_reducible() {
                    Multiply(l.clone(), box r.reduce())
                } else {
                    Number(l.value() * r.value())
                }
            },
            LessThan(ref l, ref r) => {
                if l.is_reducible() {
                    LessThan(box l.reduce(), r.clone())
                } else if r.is_reducible() {
                    LessThan(l.clone(), box r.reduce())
                } else {
                    Boolean(l.value() < r.value())
                }
            },
            _ => fail!("type mismatch in reduce")
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

    assert_eq!("1 * 2 + 3 * 4".to_string(), format!("{}", expression));
    let red = expression.reduce();
    assert_eq!("2 + 3 * 4".to_string(), format!("{}", red));
    let red = red.reduce();
    assert_eq!("2 + 12".to_string(), format!("{}", red));
    let red = red.reduce();
    assert_eq!("14".to_string(), format!("{}", red));
    assert_eq!(false, red.is_reducible())
}

pub struct Machine {
    expression: Box<Element>
}

impl Machine {
    pub fn new(expression: Box<Element>) -> Machine {
        Machine { expression: expression }
    }

    pub fn step(&mut self) {
        self.expression = box self.expression.reduce()
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
    let mut m = Machine::new(
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

    let i = box i.reduce();
    assert_eq!("true".to_string(), format!("{}", i));
    assert_eq!(false, i.is_reducible());
}
