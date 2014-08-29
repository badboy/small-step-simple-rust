extern crate std;

use std::fmt::Formatter;
use std::fmt::Result;

trait HasToS {
    fn to_s(&self) -> String;
}

pub struct Number {
    value: i64
}

impl Number {
    pub fn new(n: i64) -> Number {
        Number { value: n }
    }
}

impl HasToS for Number {
    pub fn to_s(&self) -> String {
        format!("{}", self.value)
    }
}

impl std::fmt::Show for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "«{}»", self.value.to_string())
    }
}

#[test]
fn test_instantiate() {
    let n = Number::new(3);
    assert_eq!(format!("{}", n), "«3»".to_string())
}
