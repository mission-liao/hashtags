use super::Tokenizer;
use super::Filter;
use super::super::error::Error;

pub struct SimpleTokenizer {
}

impl SimpleTokenizer {
    pub fn new() -> SimpleTokenizer {
        SimpleTokenizer{}
    }
}

impl Tokenizer for SimpleTokenizer {
    fn tokenize<'a>(&self, q: &'a str) -> Result<Filter<'a>, Error> {
        let mut ands: Vec<&str> = q.split(",").collect();
        let mut ors = Vec::<&str>::new();
        match ands.last() {
            Some(&i) => {
                if i.contains("|") {
                    ors = i.split("|").collect();
                    ands.pop();
                }
            },
            None => (),
        }
        Ok(Filter{
            ands,
            ors,
        })
    }
}

#[cfg(test)]
mod test {
    use super::SimpleTokenizer;
    use super::Tokenizer;

    #[test]
    fn test_basic() {
        let t = SimpleTokenizer::new();
        let f = t.tokenize("a,b,c,d").unwrap();
        assert_eq!(f.ands, vec!["a", "b", "c", "d"]);
        let f = t.tokenize("a,b,c|d|e").unwrap();
        assert_eq!(f.ands, vec!["a", "b"]);
        assert_eq!(f.ors, vec!["c", "d", "e"]);
    }

    #[test]
    fn test_utf8() {
        let t = SimpleTokenizer::new();
        let f = t.tokenize("台積電,聯電,ETF").unwrap();
        assert_eq!(f.ands, vec!["台積電", "聯電", "ETF"]);
        let f = t.tokenize("台積電,聯電,ETF|台達電|華碩").unwrap();
        assert_eq!(f.ands, vec!["台積電", "聯電"]);
        assert_eq!(f.ors, vec!["ETF", "台達電", "華碩"]);
    }
}
