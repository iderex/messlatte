//! The array container, against the reference implementation and against the
//! files it refuses (#35).
//!
//! Every fixture here is either produced by this writer or built by patching
//! bytes of one that was, and the patches keep the header the same length so
//! that the declared header length stays honest and the case proves the thing
//! it names rather than a truncation.

use messlatte_formats::npy::Array;

/// The reference file, base64 encoded.
///
/// It is the whole file rather than its header, and it is encoded rather than
/// written out as bytes because a raw literal in a tracked file is normalised
/// on the way into git, and this fixture exists precisely for its bytes.
///
/// What produced it, on numpy 2.5.1:
///
///     python -c "
///     import io, numpy as np, base64
///     a = np.array([[1.0,2.0,3.0],[4.0,5.0,6.5]], dtype='<f8')
///     b = io.BytesIO(); np.save(b, a, allow_pickle=False)
///     print(base64.b64encode(b.getvalue()).decode())"
const REFERENCE: &str = "k05VTVBZAQB2AHsnZGVzY3InOiAnPGY4JywgJ2ZvcnRyYW5fb3JkZXInOiBGYWxzZSwgJ3NoYXBlJzog\
                         KDIsIDMpLCB9ICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAg\
                         ICAgICAgIAoAAAAAAADwPwAAAAAAAABAAAAAAAAACEAAAAAAAAAQQAAAAAAAABRAAAAAAAAAGkA=";

/// The reference array, as this workspace builds it.
fn reference() -> Array {
    Array::new(2, 3, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.5]).expect("the shape holds six values")
}

#[test]
fn the_bytes_are_the_ones_the_reference_implementation_writes() {
    // This is the whole claim the format choice rests on. If the padding, the
    // key order or the declared length differed, the file would still read back
    // here and would hash differently from the one anybody else writes, and the
    // case index in #39 would be comparing this repository with itself.
    let written = reference().to_bytes().expect("the array is writable");
    assert_eq!(written, b64(REFERENCE), "the bytes differ from numpy's");
    assert_eq!(written.len(), 176);
}

#[test]
fn what_the_reference_implementation_wrote_reads_back_as_the_same_array() {
    let read = Array::from_bytes(&b64(REFERENCE)).expect("the reference file is readable");
    assert_eq!(read, reference());
}

#[test]
fn an_array_survives_a_round_trip_bit_for_bit() {
    // Bits rather than values, because the point of a double-precision
    // container is that nothing is rounded on the way through it, and equality
    // on floats would not see a value that came back as its own neighbour.
    let original = Array::new(1, 4, vec![0.0, -0.0, 1.0 / 3.0, f64::MIN_POSITIVE])
        .expect("four values in one row");
    let read = Array::from_bytes(&original.to_bytes().expect("writable"))
        .expect("what this wrote is readable");
    let bits: Vec<u64> = read.values.iter().map(|value| value.to_bits()).collect();
    let expected: Vec<u64> = original
        .values
        .iter()
        .map(|value| value.to_bits())
        .collect();
    assert_eq!(bits, expected);
}

#[test]
fn a_cell_off_the_array_is_nothing_rather_than_a_neighbour() {
    let array = reference();
    assert_eq!(array.at(1, 2).map(f64::to_bits), Some(6.5f64.to_bits()));
    assert_eq!(array.at(2, 0), None);
    assert_eq!(array.at(0, 3), None);
}

#[test]
fn a_shape_the_values_do_not_fill_is_refused() {
    let refusal = Array::new(2, 3, vec![1.0]).expect_err("five values are missing");
    assert!(refusal.contains("needs 6 values"), "{refusal}");
}

#[test]
fn fortran_order_is_refused_rather_than_transposed() {
    // The near-miss: `True` and `False` differ by one character and the patch
    // keeps the header length, so nothing but the ordering has changed.
    let refused = Array::from_bytes(&patched("False", "True ")).expect_err("Fortran order");
    assert!(refused.contains("Fortran order"), "{refused}");
}

#[test]
fn a_dtype_this_cannot_hold_exactly_is_refused() {
    // Big-endian doubles carry the same values and none of the bytes, and a
    // reader that ignored the byte order mark would return numbers that are
    // wrong by many orders of magnitude rather than by a rounding.
    let refused = Array::from_bytes(&patched("'<f8'", "'>f8'")).expect_err("big endian");
    assert!(refused.contains("\">f8\""), "{refused}");
}

#[test]
fn a_later_npy_version_is_refused_rather_than_read_as_this_one() {
    let mut bytes = b64(REFERENCE);
    bytes[6] = 2;
    let refused = Array::from_bytes(&bytes).expect_err("version 2.0");
    assert!(refused.contains("version 2.0"), "{refused}");
}

#[test]
fn an_array_of_one_dimension_is_refused() {
    // Six values in one dimension is the same payload as two rows of three, so
    // a reader that took the length on trust would accept this and be wrong
    // about which axis is which.
    let refused = Array::from_bytes(&patched("(2, 3)", "(6,)  ")).expect_err("one dimension");
    assert!(refused.contains("lists 1 of them"), "{refused}");
}

#[test]
fn bytes_after_the_values_are_refused() {
    let mut bytes = b64(REFERENCE);
    bytes.push(0);
    let refused = Array::from_bytes(&bytes).expect_err("a trailing byte");
    assert!(refused.contains("needs 48 bytes"), "{refused}");
}

#[test]
fn a_file_that_stops_inside_its_header_is_refused() {
    let bytes = b64(REFERENCE);
    let refused = Array::from_bytes(&bytes[..64]).expect_err("a truncated header");
    assert!(refused.contains("declares 118 bytes"), "{refused}");
}

#[test]
fn something_that_is_not_an_npy_file_is_refused() {
    let refused = Array::from_bytes(b"PK\x03\x04not an array at all")
        .expect_err("a zip archive is not an array");
    assert!(refused.contains("NPY magic"), "{refused}");
}

/// The reference file with one run of bytes replaced by another of the same
/// length.
fn patched(from: &str, to: &str) -> Vec<u8> {
    assert_eq!(
        from.len(),
        to.len(),
        "the patch has to keep the header length"
    );
    let bytes = b64(REFERENCE);
    let at = bytes
        .windows(from.len())
        .position(|window| window == from.as_bytes())
        .expect("the reference header carries the text being patched");
    let mut patched = bytes;
    patched[at..at + to.len()].copy_from_slice(to.as_bytes());
    patched
}

/// Base64, decoded here rather than depended on.
///
/// A fixture whose bytes are the point cannot be a raw literal in a tracked
/// file, and pulling in a crate to decode three constants would be a dependency
/// this workspace carries for its tests alone.
fn b64(text: &str) -> Vec<u8> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = Vec::new();
    let mut accumulator: u32 = 0;
    let mut bits = 0;
    for byte in text.bytes().filter(|byte| !byte.is_ascii_whitespace()) {
        if byte == b'=' {
            break;
        }
        let value = ALPHABET
            .iter()
            .position(|candidate| *candidate == byte)
            .unwrap_or_else(|| panic!("{:?} is not a base64 digit", byte as char));
        let value = u32::try_from(value).expect("a base64 digit is below 64");
        accumulator = (accumulator << 6) | value;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            let octet = (accumulator >> bits) & 0xff;
            out.push(u8::try_from(octet).expect("eight bits are a byte"));
        }
    }
    out
}
