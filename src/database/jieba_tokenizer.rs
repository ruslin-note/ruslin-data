use any_ascii::any_ascii_char;
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

use super::sqlite3_fts5::{SqliteError, TokenizeReason, Tokenizer};

/// My own tokenizer
///
/// The operations the tokenizer performs, in order:
/// 1. Splits data on Unicode-defined words (`UnicodeSegmentation::unicode_word_indices`).
/// 2. Converts the words to nfkc.
/// 3. Converts the words to ascii using `any_ascii`
/// 4. Ascii-lowercases the words
/// 5. Stems the word using the porter algorithm.
///
/// Should be fairly Unicode-aware whilen retaining searchability on a US keyboard.
pub struct ColTokenizer;

impl Tokenizer for ColTokenizer {
    type Global = ();

    fn new(&(): &Self::Global, _args: Vec<String>) -> Result<Self, SqliteError> {
        Ok(Self)
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
        let mut ascii_buffer = String::new();
        let mut stemmed_buffer = String::new();
        for (i, word) in text.unicode_word_indices() {
            let range = i..i + word.len();

            ascii_buffer.clear();
            ascii_buffer.extend(word.nfkc().map(any_ascii_char));
            ascii_buffer.make_ascii_lowercase();

            let graphemes = ascii_buffer.graphemes(true).collect();
            let graphemes = porter_stemmer::stem_tokenized(graphemes);
            stemmed_buffer.clear();
            stemmed_buffer.extend(graphemes.into_iter());

            (push_token)(stemmed_buffer.as_bytes(), range, false)?;
        }
        Ok(())
    }
}
