use std::io;
use std::io::{Error, ErrorKind};
use std::collections::BTreeSet;
use crate::SexprTree::{Sym, Sub};

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

#[derive(Debug,Clone,Eq, PartialEq)]
pub enum SexprTree {
    Sym(String),
    Sub(Vec<SexprTree>)
}

impl SexprTree {
    pub fn is(&self, target: &str) ->  bool {
        match self {
            Sub(_) => false,
            Sym(s) => s == target
        }
    }

    pub fn head(&self) -> Option<String> {
        match self {
            Sym(s) => Some(s.clone()),
            Sub(v) => v.get(0).and_then(|s| s.head())
        }
    }

    pub fn flatten(&self) -> Vec<String> {
        let mut result = Vec::new();
        self.flatten_help(&mut result);
        result
    }

    fn flatten_help(&self, flattened: &mut Vec<String>) {
        match self {
            Sym(s) => flattened.push(s.clone()),
            Sub(v) => v.iter().for_each(|s| s.flatten_help(flattened))
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

    pub fn build_parse_tree(src: &str) -> io::Result<SexprTree> {
        let mut parser = Parser::new(src);
        parser.tree_help()
    }

    fn tree_help(&mut self) -> io::Result<SexprTree> {
        if self.finished() {
            Ok(SexprTree::Sub(vec![]))
        } else if self.token()? == "(" {
            let mut parts = Vec::new();
            self.advance();
            while !self.at_close()? {
                parts.push(self.tree_help()?);
            }
            self.advance();
            Ok(SexprTree::Sub(parts))
        } else {
            Ok(SexprTree::Sym(self.snag()?))
        }
    }

    pub fn finished(&self) -> bool {
        self.i == self.tokens.len()
    }

    pub fn token(&self) -> io::Result<&str> {
        self.lookahead(0)
    }

    pub fn lookahead(&self, distance: usize) -> io::Result<&str> {
        let index = self.i + distance;
        match self.tokens.get(index) {
            Some(s) => Ok(s.as_str()),
            None => errorize(format!("Token index '{}'; {} tokens available", index, self.tokens.len()))
        }
    }

    pub fn check(&mut self, target_token: &str) -> io::Result<()> {
        let actual = self.token()?;
        if actual == target_token {
            self.advance();
            Ok(())
        } else {
            errorize(format!("Token '{}' expected, token '{}' encountered at position {}", target_token, actual, self.i))
        }
    }

    pub fn advance(&mut self) {
        self.advance_by(1);
    }

    pub fn advance_by(&mut self, distance: usize) {
        self.i += distance;
    }

    pub fn at_close(&self) -> io::Result<bool> {
        Ok(self.token()? == ")")
    }

    pub fn snag_symbols(&mut self) -> io::Result<Vec<String>> {
        self.check("(")?;
        let mut result = Vec::new();
        while !self.at_close()? {
            result.push(self.snag()?);
        }
        self.check(")")?;
        Ok(result)
    }

    pub fn snag(&mut self) -> io::Result<String> {
        let token = self.token()?;
        let result = String::from(token);
        self.advance();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::Parser;
    use std::io;
    use crate::SexprTree::{Sym, Sub};

    const TEST_1: &str = "(+ (* 2 3) (- 5 4))";

    #[test]
    fn token_test() {
        let tokens_1: Vec<&str> = vec!["(", "+", "(", "*", "2", "3", ")", "(", "-", "5", "4", ")", ")"];

        snag_test(&tokens_1);
        check_test(&tokens_1);
        lookahead_test(&tokens_1);
    }

    fn snag_test(tokens: &Vec<&str>) {
        let mut p = Parser::new(TEST_1);
        for token in tokens.iter() {
            assert_eq!(*token, p.snag().unwrap().as_str());
        }
        assert!(p.finished());
    }

    fn check_test(tokens: &Vec<&str>) {
        let mut p2 = Parser::new(TEST_1);
        for token in tokens.iter() {
            p2.check(*token).unwrap();
        }
        assert!(p2.finished());
    }

    fn lookahead_test(tokens: &Vec<&str>) {
        let mut p = Parser::new(TEST_1);
        for i in 0..tokens.len() - 1 {
            assert_eq!(tokens[i], p.token().unwrap());
            assert_eq!(tokens[i+1], p.lookahead(1).unwrap());
            p.advance();
        }
        p.check(")").unwrap()
    }

    #[test]
    fn snag_symbols_test() {
        let mut p = Parser::new(TEST_1);
        p.check("(").unwrap();
        p.check("+").unwrap();
        assert_eq!(p.snag_symbols().unwrap(), vec!["*", "2", "3"]);
        assert_eq!(p.snag_symbols().unwrap(), vec!["-", "5", "4"]);
        assert!(p.at_close().unwrap());
        p.check(")").unwrap();
        assert!(p.finished());
    }

    #[test]
    fn tree_test() -> io::Result<()> {
        let tree = Parser::build_parse_tree(TEST_1)?;
        match &tree {
            Sym(_) => assert!(false),
            Sub(v) => {
                assert!(v[0].is("+"));
                assert_eq!(v[1].head().unwrap().as_str(), "*");
                assert_eq!(v[2].head().unwrap().as_str(), "-");
            }
        }
        assert_eq!(format!("{:?}", tree), r#"Sub([Sym("+"), Sub([Sym("*"), Sym("2"), Sym("3")]), Sub([Sym("-"), Sym("5"), Sym("4")])])"#);
        assert_eq!(tree.head().unwrap().as_str(), "+");
        assert_eq!(tree.flatten(), vec!["+", "*", "2", "3", "-", "5", "4"]);
        Ok(())
    }
}
