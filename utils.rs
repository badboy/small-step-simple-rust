/// Write string to stderr
#![allow(unused_must_use)]
use std::io;

let mut stderr = io::stderr();
stderr.write(b"Goodbye, World!\n");
stderr.write(format!("Hello {}\n", "World").as_bytes());
