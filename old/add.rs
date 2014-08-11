extern crate std;

use std::fmt::Formatter;
use std::fmt::Result;

pub struct Add<T> {
    left: T,
    right: T
}

impl<T: std::fmt::Show> Add<T> {
    pub fn new(l: T, r: T) -> Add<T> {
        Add { left: l, right: r }
    }
}
impl HasToS for Add<T> {
    pub fn to_s(&self) -> String {
        format!("{} + {}", self.left.to_s(), self.right.to_s())
    }
}

impl<T: std::fmt::Show> std::fmt::Show for Add<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "«{}»", self.to_s())
    }
}

#[test]
fn test_instantiate() {
}
