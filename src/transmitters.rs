pub mod dcf77;
pub mod msf;

#[cfg(test)]
mod tests {
    use std::ops::Add;

    pub(crate) fn parse_expected_log(exp_str: &str) -> Vec<String> {
        exp_str
            .lines()
            .map(String::from)
            .filter(|x| !x.starts_with("//"))
            .map(|x| x.add("\n"))
            .collect()
    }
}
