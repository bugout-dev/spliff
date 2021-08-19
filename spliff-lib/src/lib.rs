pub mod account;
pub mod errors;
pub mod state;
pub mod token;
pub fn hello() -> String {
    "Hello".to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
