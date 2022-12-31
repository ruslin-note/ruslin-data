mod unicode_segmentation_tables;

use super::sqlite3_fts5::{SqliteError, TokenizeReason, Tokenizer};
use jieba_rs::Jieba;

pub struct JiebaTokenizer(JiebaTokenizerImpl);

impl Tokenizer for JiebaTokenizer {
    type Global = ();

    fn new(&(): &Self::Global, _args: Vec<String>) -> Result<Self, SqliteError> {
        Ok(Self(JiebaTokenizerImpl::new()))
    }

    fn tokenize<TKF>(
        &mut self,
        _reason: TokenizeReason,
        text: &[u8],
        mut push_token: TKF,
    ) -> Result<(), SqliteError>
    where
        TKF: FnMut(&[u8], std::ops::Range<usize>, bool) -> Result<(), SqliteError>,
    {
        let text = String::from_utf8_lossy(text);
        let tokens = self.0.tokenize(&text);
        for token in tokens {
            let range = token.start..token.end;
            push_token(token.word.as_bytes(), range, false)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token<'a> {
    /// Word of the token
    pub word: &'a str,
    /// Bytes start position of the token
    pub start: usize,
    /// Bytes end position of the token
    pub end: usize,
}

pub struct JiebaTokenizerImpl(Jieba);

impl JiebaTokenizerImpl {
    pub fn new() -> Self {
        Self(Jieba::new())
    }

    pub fn tokenize<'a>(&self, sentence: &'a str) -> Vec<Token<'a>> {
        let words = self.0.cut(sentence, false);
        let mut tokens = Vec::with_capacity(words.len());
        let mut start = 0;
        for word in words {
            let width = word.len();
            if has_alphanumeric(word) {
                tokens.push(Token {
                    word,
                    start,
                    end: start + width,
                });
            }
            start += width;
        }
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::JiebaTokenizerImpl;

    #[test]
    fn test_jieba_tokenizer_impl() {
        let jieba_tokenizer_impl = JiebaTokenizerImpl::new();
        let text = String::from_utf8_lossy(
            "我是拖拉机学院手扶拖拉机专业的。不用多久，我就会升职加薪，当上CEO，走上人生巅峰。"
                .as_bytes(),
        );
        let tokens = jieba_tokenizer_impl.tokenize(&text);
        for token in tokens {
            assert_eq!(token.word, &text[token.start..token.end]);
        }
    }
}

#[inline]
fn has_alphanumeric(s: &str) -> bool {
    use unicode_segmentation_tables::util::is_alphanumeric;

    s.chars().any(is_alphanumeric)
}
