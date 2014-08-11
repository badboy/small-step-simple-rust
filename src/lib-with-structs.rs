extern crate std;

use std::fmt::Formatter;
use std::fmt::Result;

pub trait Element {
    fn to_s(&self) -> String;

    fn is_reducible(&self) -> bool;
    fn reduce(&self) -> Self;
}

pub struct Number {
    value: i64
}

impl Number {
    pub fn new(n: i64) -> Number {
        Number { value: n }
    }
}

impl Element for Number {
    fn to_s(&self) -> String {
        format!("{}", self.value)
    }

    fn is_reducible(&self) -> bool {
        false
    }

    fn reduce(&self) -> Number {
        Number::new(self.value)
    }
}

impl std::fmt::Show for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "«{}»", self.value.to_string())
    }
}

#[test]
fn test_instantiate_number() {
    let n = Number::new(3);
    assert_eq!(format!("{}", n), "«3»".to_string())
}

pub struct Add<L, R> {
    left: L,
    right: R
}

impl<L: Element, R: Element> Add<L,R> {
    pub fn new(l: L, r: R) -> Add<L,R> {
        Add { left: l, right: r }
    }
}

impl<L: Element, R: Element> Element for Add<L,R> {
    fn to_s(&self) -> String {
        format!("{} + {}", self.left.to_s(), self.right.to_s())
    }

    fn is_reducible(&self) -> bool {
        true
    }

    fn reduce(&self) -> Add<L,R> {
        Add::new(L::new(1), Number::new(0))
        //if self.left.is_reducible() {
            //Add::new(self.left.reduce(), self.right)
        //} else if self.right.is_reducible() {
            //Add::new(self.left, self.right.reduce())
        //} else {
            //Add::new(Number::new(1), Number::new(0))
        //}
    }
}

impl<L: Element, R: Element> std::fmt::Show for Add<L,R> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "«{}»", self.to_s())
    }
}

#[test]
fn test_instantiate_add() {
    let a = Add::new(Number::new(3), Number::new(4));
    assert_eq!(format!("{}", a), "«3 + 4»".to_string())
}

pub struct Multiply<L,R> {
    left: L,
    right: R
}

impl<L: Element, R: Element> Multiply<L,R> {
    pub fn new(l: L, r: R) -> Multiply<L,R> {
        Multiply { left: l, right: r }
    }
}

impl<L: Element, R: Element> Element for Multiply<L,R> {
    fn to_s(&self) -> String {
        format!("{} * {}", self.left.to_s(), self.right.to_s())
    }

    fn is_reducible(&self) -> bool {
        true
    }

    fn reduce(&self) -> Multiply<L,R> {
        Multiply::new(self.left, self.right)
    }
}

impl<L: Element, R: Element> std::fmt::Show for Multiply<L,R> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "«{}»", self.to_s())
    }
}

#[test]
fn test_instantiate_multiply() {
    let a = Multiply::new(Number::new(3), Number::new(4));
    assert_eq!(format!("{}", a), "«3 * 4»".to_string())
}
