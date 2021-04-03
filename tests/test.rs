#[cfg(test)]
mod tests {
    use ::caked::deser_str;
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn abel() {
        let cu = deser_str("<?php [ 'god' => 'willing', 'abc' => 'def', 'ghi' => [ 'jkl' => 1.0, 2.0 ] ]").unwrap();
        assert_eq!(cu.len(), 3);
    }
}
