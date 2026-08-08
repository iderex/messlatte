//! JSON, restricted to what a header document needs and written in one order.
//!
//! Two properties are wanted and neither comes from a general library for free.
//! A document written from the same content twice is the same bytes, so a
//! header hashes stably and a diff between two cases reads as a change in the
//! case rather than as a reformatting. And a document read and written again is
//! the same bytes, so a reader that has to preserve a field it does not
//! understand can do it by holding the parsed value rather than by holding the
//! original text.
//!
//! The canonical order is the byte order of the keys, which [`Object`] gets
//! from `BTreeMap`. It is chosen because it is the only order a second
//! implementation can reproduce without being told the schema, and the schema
//! is the thing that changes.
//!
//! A number has one canonical spelling too, and it is the shortest decimal that
//! reads back as the same double, written without an exponent. The exponent
//! form has several spellings of one value and the plain form has one, so the
//! plain form is what a second implementation can agree with. What it costs is
//! that a very small number is written out in full, which is long rather than
//! wrong: a delay in seconds takes a few more characters than it would in
//! exponent form and reads back as the same double either way. A document
//! written elsewhere is normalised into this form on the way through rather
//! than echoed, so what is hashed is what the document means.
//!
//! What this is not. It is a reader for a header of a few kilobytes, it holds
//! the whole document in memory, and it recurses once per nesting level, so a
//! deeply nested document is refused by depth rather than by exhausting the
//! stack. It is not a general-purpose JSON library and nothing here should grow
//! into one: the moment a format needs more than this, the question is whether
//! the format needs it.

use std::collections::BTreeMap;
use std::fmt::Write as _;

/// The members of an object, in the order they are written.
pub type Object = BTreeMap<String, Json>;

/// How deep a document may nest.
///
/// The parser descends one stack frame per level, so without a bound a
/// document made of ten thousand open brackets ends the process rather than
/// returning an error. A header is two levels deep and the third is already an
/// argument about the format, so this is far above anything legitimate and far
/// below what a stack costs.
const MAX_DEPTH: usize = 32;

/// A JSON value.
///
/// A number is a `f64`, which is what the arrays in a trace header are, and
/// what that costs is worth stating: an integer beyond the exactly
/// representable range is read back as the nearest double. Nothing in these
/// formats carries such an integer, and a format that needs one carries it as a
/// string.
#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Object(Object),
}

impl Json {
    /// The member of an object under one key, or nothing.
    ///
    /// Returns nothing for a value that is not an object as well, because every
    /// caller here is asking whether the document carries a field and both
    /// answers to that are the same answer.
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Object(members) => members.get(key),
            _ => None,
        }
    }

    /// The value as a string, or nothing.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::String(text) => Some(text),
            _ => None,
        }
    }

    /// The value as a number, or nothing.
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Json::Number(value) => Some(*value),
            _ => None,
        }
    }

    /// The value as an array, or nothing.
    pub fn as_array(&self) -> Option<&[Json]> {
        match self {
            Json::Array(items) => Some(items),
            _ => None,
        }
    }

    /// The document as bytes, in canonical order, with no insignificant space.
    ///
    /// A trailing newline is added, so the file is a text file by the usual
    /// convention and a terminal that prints it does not run the next prompt
    /// into the closing brace.
    ///
    /// It refuses a number that is not finite. JSON has no spelling for one, a
    /// writer that emitted `NaN` would produce a document no other reader
    /// accepts, and in a trace header the value it would carry is a missing
    /// sample written as data, which is the thing the format exists to keep
    /// out.
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        let mut text = String::new();
        self.write_into(&mut text)?;
        text.push('\n');
        Ok(text.into_bytes())
    }

    fn write_into(&self, out: &mut String) -> Result<(), String> {
        match self {
            Json::Null => out.push_str("null"),
            Json::Bool(true) => out.push_str("true"),
            Json::Bool(false) => out.push_str("false"),
            Json::Number(value) => {
                if !value.is_finite() {
                    return Err(format!(
                        "the number {value} is not finite, and JSON has no spelling for it"
                    ));
                }
                // The default formatting of a double in Rust is the shortest
                // decimal that reads back as the same value, so this is exact
                // in both directions and two writes of one value agree.
                let _ = write!(out, "{value}");
            }
            Json::String(text) => write_string(text, out),
            Json::Array(items) => {
                out.push('[');
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        out.push(',');
                    }
                    item.write_into(out)?;
                }
                out.push(']');
            }
            Json::Object(members) => {
                out.push('{');
                for (index, (key, value)) in members.iter().enumerate() {
                    if index > 0 {
                        out.push(',');
                    }
                    write_string(key, out);
                    out.push(':');
                    value.write_into(out)?;
                }
                out.push('}');
            }
        }
        Ok(())
    }
}

/// One string, escaped no more than it has to be.
///
/// The quote and the backslash have to be escaped, and so does every character
/// below a space. Everything else is written as itself, including every
/// non-ASCII character, so the document is UTF-8 and a reader sees the
/// character rather than an escape of it.
fn write_string(text: &str, out: &mut String) {
    out.push('"');
    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            other if (other as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", other as u32);
            }
            other => out.push(other),
        }
    }
    out.push('"');
}

/// A document from its bytes.
///
/// The whole input has to be one value. Text after the value is refused rather
/// than ignored, because a second document appended to the first is a file
/// somebody assembled wrongly and a reader that stopped at the first would
/// score it against half its content.
pub fn parse(bytes: &[u8]) -> Result<Json, String> {
    let text = std::str::from_utf8(bytes)
        .map_err(|error| format!("the document is not UTF-8: {error}"))?;
    let mut reader = Reader {
        rest: text.chars().collect(),
        at: 0,
    };
    reader.skip_space();
    let value = reader.value(0)?;
    reader.skip_space();
    if reader.at < reader.rest.len() {
        return Err(format!(
            "the document ends at character {} and {} characters follow it",
            reader.at,
            reader.rest.len() - reader.at
        ));
    }
    Ok(value)
}

struct Reader {
    rest: Vec<char>,
    at: usize,
}

impl Reader {
    fn peek(&self) -> Option<char> {
        self.rest.get(self.at).copied()
    }

    fn skip_space(&mut self) {
        while matches!(self.peek(), Some(' ' | '\t' | '\n' | '\r')) {
            self.at += 1;
        }
    }

    fn expect(&mut self, character: char) -> Result<(), String> {
        if self.peek() == Some(character) {
            self.at += 1;
            Ok(())
        } else {
            Err(self.unexpected(&format!("{character:?}")))
        }
    }

    fn unexpected(&self, wanted: &str) -> String {
        match self.peek() {
            Some(found) => format!("character {}: expected {wanted}, found {found:?}", self.at),
            None => format!("character {}: expected {wanted}, found the end", self.at),
        }
    }

    fn literal(&mut self, word: &str) -> bool {
        if self.rest[self.at..].starts_with(&word.chars().collect::<Vec<char>>()[..]) {
            self.at += word.chars().count();
            true
        } else {
            false
        }
    }

    fn value(&mut self, depth: usize) -> Result<Json, String> {
        if depth > MAX_DEPTH {
            return Err(format!(
                "the document nests deeper than {MAX_DEPTH} levels at character {}",
                self.at
            ));
        }
        match self.peek() {
            Some('{') => self.object(depth),
            Some('[') => self.array(depth),
            Some('"') => Ok(Json::String(self.string()?)),
            Some('t') if self.literal("true") => Ok(Json::Bool(true)),
            Some('f') if self.literal("false") => Ok(Json::Bool(false)),
            Some('n') if self.literal("null") => Ok(Json::Null),
            _ => self.number(),
        }
    }

    fn object(&mut self, depth: usize) -> Result<Json, String> {
        self.expect('{')?;
        let mut members = Object::new();
        self.skip_space();
        if self.peek() == Some('}') {
            self.at += 1;
            return Ok(Json::Object(members));
        }
        loop {
            self.skip_space();
            let at = self.at;
            let key = self.string()?;
            self.skip_space();
            self.expect(':')?;
            self.skip_space();
            let value = self.value(depth + 1)?;
            // A repeated key is refused rather than resolved. Both resolutions
            // are somebody's convention, the two disagree about what the
            // document says, and a header whose meaning depends on which reader
            // opened it is the failure the format version exists against.
            if members.insert(key.clone(), value).is_some() {
                return Err(format!("character {at}: the key {key:?} appears twice"));
            }
            self.skip_space();
            match self.peek() {
                Some(',') => self.at += 1,
                Some('}') => {
                    self.at += 1;
                    return Ok(Json::Object(members));
                }
                _ => return Err(self.unexpected("',' or '}'")),
            }
        }
    }

    fn array(&mut self, depth: usize) -> Result<Json, String> {
        self.expect('[')?;
        let mut items = Vec::new();
        self.skip_space();
        if self.peek() == Some(']') {
            self.at += 1;
            return Ok(Json::Array(items));
        }
        loop {
            self.skip_space();
            items.push(self.value(depth + 1)?);
            self.skip_space();
            match self.peek() {
                Some(',') => self.at += 1,
                Some(']') => {
                    self.at += 1;
                    return Ok(Json::Array(items));
                }
                _ => return Err(self.unexpected("',' or ']'")),
            }
        }
    }

    fn string(&mut self) -> Result<String, String> {
        self.expect('"')?;
        let mut text = String::new();
        loop {
            let at = self.at;
            let character = self
                .peek()
                .ok_or_else(|| format!("character {at}: the string does not end"))?;
            self.at += 1;
            match character {
                '"' => return Ok(text),
                '\\' => text.push(self.escape()?),
                other if (other as u32) < 0x20 => {
                    return Err(format!(
                        "character {at}: a string carries a control character directly, \
                         and it has to be escaped"
                    ))
                }
                other => text.push(other),
            }
        }
    }

    fn escape(&mut self) -> Result<char, String> {
        let at = self.at;
        let character = self
            .peek()
            .ok_or_else(|| format!("character {at}: the escape does not end"))?;
        self.at += 1;
        Ok(match character {
            '"' => '"',
            '\\' => '\\',
            '/' => '/',
            'b' => '\u{8}',
            'f' => '\u{c}',
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            'u' => return self.escaped_code_point(),
            other => return Err(format!("character {at}: {other:?} is not an escape")),
        })
    }

    /// One `\u` escape, and the pair of them a character outside the basic
    /// plane is written as.
    ///
    /// The pair is handled rather than refused because a writer in another
    /// language emits it without being asked to, and a reader that refused it
    /// would reject a document its own writer would accept.
    fn escaped_code_point(&mut self) -> Result<char, String> {
        let first = self.four_hex_digits()?;
        if !(0xD800..0xDC00).contains(&first) {
            return char::from_u32(first).ok_or_else(|| {
                format!(
                    "character {}: \\u{first:04x} is not a character, and a lone half \
                     of a surrogate pair cannot be one",
                    self.at
                )
            });
        }
        let at = self.at;
        let followed_by_escape =
            self.rest.get(self.at) == Some(&'\\') && self.rest.get(self.at + 1) == Some(&'u');
        if !followed_by_escape {
            return Err(format!(
                "character {at}: \\u{first:04x} is the first half of a surrogate pair \
                 and the second half does not follow it"
            ));
        }
        self.at += 2;
        let second = self.four_hex_digits()?;
        if !(0xDC00..0xE000).contains(&second) {
            return Err(format!(
                "character {at}: \\u{first:04x} is followed by \\u{second:04x}, \
                 which is not the second half of a surrogate pair"
            ));
        }
        let code = 0x1_0000 + ((first - 0xD800) << 10) + (second - 0xDC00);
        char::from_u32(code).ok_or_else(|| {
            format!("character {at}: \\u{first:04x}\\u{second:04x} is not a character")
        })
    }

    fn four_hex_digits(&mut self) -> Result<u32, String> {
        let at = self.at;
        let mut value = 0u32;
        for _ in 0..4 {
            let digit = self
                .peek()
                .and_then(|character| character.to_digit(16))
                .ok_or_else(|| format!("character {at}: an escape needs four hex digits"))?;
            value = value * 16 + digit;
            self.at += 1;
        }
        Ok(value)
    }

    /// One number, scanned against the JSON grammar and then converted.
    ///
    /// The grammar is checked here rather than left to the conversion, because
    /// Rust reads `inf`, `NaN` and a leading `+` and JSON does not, so a
    /// document carrying one of those would be accepted by a reader that only
    /// tried to convert.
    fn number(&mut self) -> Result<Json, String> {
        let start = self.at;
        if self.peek() == Some('-') {
            self.at += 1;
        }
        let integer_start = self.at;
        let integer = self.digits();
        if integer == 0 {
            return Err(self.unexpected("a value"));
        }
        // A leading zero is refused, so `012` is not read as twelve by one
        // reader and as an error by another.
        if integer > 1 && self.rest[integer_start] == '0' {
            return Err(format!(
                "character {integer_start}: a number has no leading zero"
            ));
        }
        if self.peek() == Some('.') {
            self.at += 1;
            if self.digits() == 0 {
                return Err(self.unexpected("a digit after the decimal point"));
            }
        }
        if matches!(self.peek(), Some('e' | 'E')) {
            self.at += 1;
            if matches!(self.peek(), Some('+' | '-')) {
                self.at += 1;
            }
            if self.digits() == 0 {
                return Err(self.unexpected("a digit in the exponent"));
            }
        }
        let text: String = self.rest[start..self.at].iter().collect();
        let value: f64 = text
            .parse()
            .map_err(|error| format!("character {start}: {text:?} is not a number: {error}"))?;
        if !value.is_finite() {
            return Err(format!(
                "character {start}: {text:?} is outside the range a double holds"
            ));
        }
        Ok(Json::Number(value))
    }

    fn digits(&mut self) -> usize {
        let start = self.at;
        while matches!(self.peek(), Some(digit) if digit.is_ascii_digit()) {
            self.at += 1;
        }
        self.at - start
    }
}
