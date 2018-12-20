
mod entities;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
            let mut ii:u64 = 18446744073709551615;
        assert_eq!(ii.checked_add(1), None);
    }
}
