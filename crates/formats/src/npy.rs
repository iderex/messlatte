//! The array container: NPY version 1.0, two dimensions, double precision, C
//! order.
//!
//! The choice is argued in #35 and the short form is that a trace is one
//! two-dimensional array of doubles, and this is the smallest documented format
//! that carries exactly that. The specification is short enough to implement
//! from, the layout is the header followed by the raw values, and identical
//! content produces identical bytes, which is what a hash over a trace needs.
//!
//! What is deliberately not supported, so that a file this repository refuses
//! is refused for a reason a reader can act on rather than by falling over.
//! Fortran order is refused rather than transposed, because a trace whose
//! ordering is decided by whoever wrote it is a trace whose axes are decided by
//! whoever read it. Any dtype other than little-endian eight-byte float is
//! refused, including a bigger one, because a value this repository cannot hold
//! exactly is a value it should not silently round. Versions 2.0 and 3.0 exist
//! and are refused: they widen the header length field and admit a non-ASCII
//! header, neither of which a header of three keys needs.
//!
//! The header this writer produces is byte for byte the one the reference
//! implementation produces for the same array, which is what makes the
//! cross-reading check in #35 a check of the values rather than of the padding.

/// The first six bytes of every NPY file.
const MAGIC: &[u8] = b"\x93NUMPY";

/// The magic and the two version bytes.
const MAGIC_LEN: usize = 8;

/// The header is padded so that the values begin on a multiple of this. The
/// reference implementation aligns to it so that a memory-mapped read starts on
/// a boundary, and a writer that skipped the padding would produce a file that
/// reads correctly and hashes differently.
const ALIGN: usize = 64;

/// The one dtype this module reads and writes.
const DTYPE: &str = "<f8";

/// A two-dimensional array of doubles, held row by row.
///
/// The rows are the slower axis, which is what C order means, so `values[row *
/// columns + column]` is the cell. A trace binds those two axes to physical
/// ones and that binding is in `trace`, not here.
#[derive(Debug, Clone, PartialEq)]
pub struct Array {
    pub rows: usize,
    pub columns: usize,
    pub values: Vec<f64>,
}

impl Array {
    /// An array from its shape and its values, refusing a values slice that is
    /// not the shape's length.
    pub fn new(rows: usize, columns: usize, values: Vec<f64>) -> Result<Array, String> {
        let expected = rows
            .checked_mul(columns)
            .ok_or_else(|| format!("the shape ({rows}, {columns}) does not fit in memory"))?;
        if values.len() != expected {
            return Err(format!(
                "the shape ({rows}, {columns}) needs {expected} values and {} were given",
                values.len()
            ));
        }
        Ok(Array {
            rows,
            columns,
            values,
        })
    }

    /// One cell, or nothing where the indices are off the array.
    pub fn at(&self, row: usize, column: usize) -> Option<f64> {
        if row >= self.rows || column >= self.columns {
            return None;
        }
        self.values.get(row * self.columns + column).copied()
    }

    /// The file, header and values.
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        let text = format!(
            "{{'descr': '{DTYPE}', 'fortran_order': False, 'shape': ({}, {}), }}",
            self.rows, self.columns
        );
        // The length the field carries includes the closing newline, and the
        // padding is what brings the whole prefix onto the alignment. Where the
        // prefix already lands on it, a full block of padding is added rather
        // than none, which is what the reference implementation does and what a
        // byte comparison against it therefore requires.
        let unpadded = MAGIC_LEN + 2 + text.len() + 1;
        let padding = ALIGN - (unpadded % ALIGN);
        let declared = text.len() + 1 + padding;
        let declared = u16::try_from(declared).map_err(|_| {
            format!("the header is {declared} bytes, which version 1.0 cannot declare")
        })?;

        let mut out =
            Vec::with_capacity(MAGIC_LEN + 2 + usize::from(declared) + self.values.len() * 8);
        out.extend_from_slice(MAGIC);
        out.push(1);
        out.push(0);
        out.extend_from_slice(&declared.to_le_bytes());
        out.extend_from_slice(text.as_bytes());
        out.extend(std::iter::repeat_n(b' ', padding));
        out.push(b'\n');
        for value in &self.values {
            out.extend_from_slice(&value.to_le_bytes());
        }
        Ok(out)
    }

    /// An array from a whole file.
    ///
    /// Bytes after the values are refused rather than ignored. A file carrying
    /// more than its shape declares was written by something this reader does
    /// not understand, and reading the part it recognises would report a
    /// truncation as a trace.
    pub fn from_bytes(bytes: &[u8]) -> Result<Array, String> {
        if bytes.len() < MAGIC_LEN + 2 {
            return Err(format!(
                "the file is {} bytes, which is shorter than an NPY header",
                bytes.len()
            ));
        }
        if &bytes[..MAGIC.len()] != MAGIC {
            return Err("the file does not begin with the NPY magic".to_string());
        }
        let (major, minor) = (bytes[6], bytes[7]);
        if (major, minor) != (1, 0) {
            return Err(format!(
                "the file declares NPY version {major}.{minor}, and this reads version 1.0 only"
            ));
        }
        let declared = usize::from(u16::from_le_bytes([bytes[8], bytes[9]]));
        let start = MAGIC_LEN + 2;
        let end = start
            .checked_add(declared)
            .filter(|end| *end <= bytes.len())
            .ok_or_else(|| {
                format!(
                    "the header declares {declared} bytes and the file holds {} after the prefix",
                    bytes.len() - start
                )
            })?;
        let text = std::str::from_utf8(&bytes[start..end])
            .map_err(|error| format!("the NPY header is not text: {error}"))?;

        let descr = quoted_after(text, "'descr'")
            .ok_or_else(|| "the NPY header declares no dtype".to_string())?;
        if descr != DTYPE {
            return Err(format!(
                "the array is of dtype {descr:?}, and this reads {DTYPE:?} only"
            ));
        }
        match word_after(text, "'fortran_order'").as_deref() {
            Some("False") => {}
            Some("True") => {
                return Err(
                    "the array is in Fortran order, and this reads C order only. Rewrite it in \
                     C order rather than transposing it on the way in, so that the axes a \
                     reader sees are the axes the writer meant"
                        .to_string(),
                )
            }
            other => {
                return Err(format!(
                    "the NPY header declares a Fortran order of {other:?}, which is neither \
                     True nor False"
                ))
            }
        }
        let shape = shape_after(text)?;
        let [rows, columns] = shape[..] else {
            return Err(format!(
                "a trace array has two dimensions and this shape lists {} of them",
                shape.len()
            ));
        };

        let payload = &bytes[end..];
        let needed = rows
            .checked_mul(columns)
            .and_then(|cells| cells.checked_mul(8))
            .ok_or_else(|| format!("the shape ({rows}, {columns}) does not fit in memory"))?;
        if payload.len() != needed {
            return Err(format!(
                "the shape ({rows}, {columns}) needs {needed} bytes of values and the file \
                 holds {}",
                payload.len()
            ));
        }
        let mut values = Vec::with_capacity(payload.len() / 8);
        for chunk in payload.chunks_exact(8) {
            let mut eight = [0u8; 8];
            eight.copy_from_slice(chunk);
            values.push(f64::from_le_bytes(eight));
        }
        Array::new(rows, columns, values)
    }
}

/// The quoted value after a key in the header dictionary.
///
/// The header is a Python dictionary literal rather than a document format, and
/// this reads the three keys it needs out of it instead of parsing the
/// language. What that costs is worth stating: a key inside a string value
/// elsewhere in the header would be found by this, and a header written with a
/// different quoting convention would not. The three keys are the only ones the
/// specification defines, and a header carrying a fourth is not something this
/// reader claims to understand.
fn quoted_after(text: &str, key: &str) -> Option<String> {
    let rest = text.split_once(key)?.1;
    let rest = rest.split_once(':')?.1;
    let rest = rest.trim_start();
    let mut characters = rest.chars();
    let quote = characters.next()?;
    if quote != '\'' && quote != '"' {
        return None;
    }
    let value: String = characters
        .take_while(|character| *character != quote)
        .collect();
    Some(value)
}

/// The bare word after a key, which is how `True` and `False` are written.
fn word_after(text: &str, key: &str) -> Option<String> {
    let rest = text.split_once(key)?.1;
    let rest = rest.split_once(':')?.1;
    Some(
        rest.trim_start()
            .chars()
            .take_while(|character| character.is_ascii_alphanumeric())
            .collect(),
    )
}

/// The shape tuple, as the dimensions it lists.
fn shape_after(text: &str) -> Result<Vec<usize>, String> {
    let rest = text
        .split_once("'shape'")
        .and_then(|(_, rest)| rest.split_once(':'))
        .map(|(_, rest)| rest)
        .ok_or_else(|| "the NPY header declares no shape".to_string())?;
    let inside = rest
        .trim_start()
        .strip_prefix('(')
        .and_then(|rest| rest.split_once(')'))
        .map(|(inside, _)| inside)
        .ok_or_else(|| "the NPY header's shape is not a tuple".to_string())?;
    let mut dimensions = Vec::new();
    for part in inside.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        dimensions.push(part.parse::<usize>().map_err(|error| {
            format!("the shape carries {part:?}, which is not a length: {error}")
        })?);
    }
    Ok(dimensions)
}
