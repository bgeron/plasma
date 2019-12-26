//! A library for Ion de/serialization; in particular the binary encoding.

pub mod binary;
mod imports;

pub fn five() -> i32 {
    5
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
