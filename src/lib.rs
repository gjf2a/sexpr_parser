use std::io;
use std::io::{Error, ErrorKind};
use std::collections::BTreeSet;

pub fn errorize<T>(msg: String) -> io::Result<T> {
    Err(Error::new(ErrorKind::Other, msg.as_str()))
}

struct Tokenizer {
    pending: String,
    symbols: BTreeSet<char>
}

impl Tokenizer {
    pub fn new(symbols: &str) -> Self {
        let mut result = Tokenizer {pending: String::new(), symbols: BTreeSet::new()};
        symbols.chars().for_each(|c| {result.symbols.insert(c);});
        result
    }

    pub fn tokenize(&mut self, text: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        text.chars().for_each(|c| {
            if self.symbols.contains(&c) {
                self.add_pending(&mut tokens);
                let mut cstr = String::new();
                cstr.push(c);
                tokens.push(cstr);
            } else if c.is_whitespace() {
                self.add_pending(&mut tokens);
            } else {
                self.pending.push(c);
            }
        });
        self.add_pending(&mut tokens);
        tokens
    }

    fn add_pending(&mut self, tokens: &mut Vec<String>) {
        if self.pending.len() > 0 {
            tokens.push(self.pending.to_lowercase());
            self.pending = String::new();
        }
    }
}

pub struct Parser {
    tokens: Vec<String>,
    i: usize
}

impl Parser {
    pub fn new(src: &str) -> Self {
        Parser {tokens: Tokenizer::new("()").tokenize(src), i: 0}
    }

    fn token(&self) -> io::Result<&str> {
        self.lookahead(0)
    }

    fn lookahead(&self, distance: usize) -> io::Result<&str> {
        let index = self.i + distance;
        match self.tokens.get(index) {
            Some(s) => Ok(s.as_str()),
            None => errorize(format!("Token index '{}'; {} tokens available", index, self.tokens.len()))
        }
    }

    fn check(&mut self, target_token: &str) -> io::Result<()> {
        let actual = self.token()?;
        if actual == target_token {
            self.advance();
            Ok(())
        } else {
            errorize(format!("Token '{}' expected, token '{}' encountered at position {}", target_token, actual, self.i))
        }
    }

    fn advance(&mut self) {
        self.advance_by(1);
    }

    fn advance_by(&mut self, distance: usize) {
        self.i += distance;
    }

    fn at_close(&self) -> io::Result<bool> {
        Ok(self.token()? == ")")
    }

    fn snag_symbols(&mut self) -> io::Result<Vec<String>> {
        self.check("(")?;
        let mut result = Vec::new();
        while !self.at_close()? {
            result.push(self.snag()?);
        }
        self.check(")")?;
        Ok(result)
    }

    fn snag(&mut self) -> io::Result<String> {
        let token = self.token()?;
        let result = String::from(token);
        self.advance();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
