///
/// https://www.sqlite.org/datatype3.html
/// Each value stored in an SQLite database (or manipulated by the database engine) has one of the following storage classes:
///   NULL. The value is a NULL value.
///   INTEGER. The value is a signed integer,
///     stored in 0, 1, 2, 3, 4, 6, or 8 bytes depending on the magnitude of the value.
///   REAL. The value is a floating point value, stored as an 8-byte IEEE
///     floating point number.
///   TEXT. The value is a text string, stored using the
///     database encoding (UTF-8, UTF-16BE or UTF-16LE).
///   BLOB. The value is a blob of data, stored exactly as it was input.
///
/// Any column in an SQLite version 3 database, except an INTEGER PRIMARY KEY column,
/// may be used to store a value of any storage class.
use anyhow::Result;
use std::str::from_utf8;
//
// #[derive(Debug)]
// enum ColumnType {
//     Null,
//     // integer
//     I8,
//     I16,
//     I24,
//     I32,
//     I48,
//     I64,
//     // real
//     F64,
//     Text(usize),
//     Blob(usize),
// }

// https://github.com/richardmarbach/codecrafters-sqlite-rust/blob/master/src/record.rs#L110
// https://github.com/bert2/build-your-own-sqlite-rust/blob/master/src/format/col_content.rs
// TODO optimize - copy for string and bytes values are costly, ok for the rest (int, float,...)
#[derive(Debug, PartialEq, Clone)]
pub enum ColumnValue {
    Null,
    Int8([u8; 1]),
    Int16([u8; 2]),
    Int24([u8; 3]),
    // Int32([u8; 4]),
    Int32([u8; 4]),
    Int48([u8; 6]),
    Int64([u8; 8]),
    Float64([u8; 8]),
    Zero,
    One,
    // 2 options for bytes of Blob: use &[u8] or Box<[u8]> because we don't know the size
    // of byte array at compile time so we need a pointer (reference).
    // Because the bytes of blob and text are in the db file bytes already and no need to modify,
    // let's ref to that with a shared reference &[u8].
    // We need a lifetime specifier param 'a to tell the compiler that the lifetime of
    // u8 slices of Blob or Text have the same lifetime with the owning ColumnValue value.
    // Blob(&'a [u8]),

    // switching from &'a [u8] to Box<u8> because I don't want to specify lifetime 'a.
    // Bytes of Blob and Text are created on the heap.
    Blob(Vec<u8>),
    Text(String),
}

impl ColumnValue {
    pub fn int32(int: i32) -> ColumnValue {
        ColumnValue::Int32(int.to_be_bytes())
    }
    pub fn int8(int: i8) -> ColumnValue {
        ColumnValue::Int8(int.to_be_bytes())
    }

    /// Parses column value from bytes.
    /// https://www.sqlite.org/fileformat.html#record_format
    pub fn parse(serial_type: i64, stream: &[u8]) -> Result<(ColumnValue, usize)> {
        Ok(match serial_type {
            0 => (ColumnValue::Null, 0),
            1 => (ColumnValue::Int8(stream[..1].try_into()?), 1),
            2 => (ColumnValue::Int16(stream[..2].try_into()?), 2),
            3 => (ColumnValue::Int24(stream[..3].try_into()?), 3),
            4 => (ColumnValue::Int32(stream[..4].try_into()?), 4),
            5 => (ColumnValue::Int48(stream[..6].try_into()?), 6),
            6 => (ColumnValue::Int64(stream[..8].try_into()?), 8),
            7 => (ColumnValue::Float64(stream[..8].try_into()?), 8),
            8 => (ColumnValue::Zero, 0),
            9 => (ColumnValue::One, 0),
            n if n >= 12 && n % 2 == 0 => {
                let len = ((n - 12) / 2).try_into()?;
                (ColumnValue::Blob(stream[..len].try_into()?), len)
            }
            n if n >= 13 && n % 2 == 1 => {
                let len = ((n - 13) / 2).try_into()?;
                // This function returns a Cow<'a, str>. If our byte slice is invalid UTF-8,
                // then we need to insert the replacement characters, which will change
                // the size of the string, and hence, require a String.
                // But if it’s already valid UTF-8, we don’t need a new allocation.
                // This return type allows us to handle both cases.
                //
                // Use from_utf8_unchecked for faster perf without the check overhead
                let value = String::from_utf8_lossy(&stream[..len]).to_string();
                (ColumnValue::Text(value), len)
            }
            n => panic!("Invalid serial type: {}", n),
        })
    }
}

#[test]
fn test_parse_col_value_null() {
    let (value, size) = ColumnValue::parse(0, b"").unwrap();
    assert_eq!(size, 0);
    assert_eq!(value, ColumnValue::Null);
}

#[test]
fn test_parse_col_value_zero() {
    let (value, size) = ColumnValue::parse(8, b"123").unwrap();
    assert_eq!(size, 0);
    assert_eq!(value, ColumnValue::Zero);
}

#[test]
fn test_parse_col_value_one() {
    let (value, size) = ColumnValue::parse(9, b"33").unwrap();
    assert_eq!(size, 0);
    assert_eq!(value, ColumnValue::One);
}

#[test]
fn test_parse_col_value_blob() {
    // one byte br'0000_0001' = b'1' -> len 1 * 2 + 12 = 14
    // should parse only first byte, ignore second byte
    let (value, size) = ColumnValue::parse(14, b"12").unwrap();
    assert_eq!(size, 1);
    assert_eq!(value, ColumnValue::Blob(vec!(b'1')));
}

#[test]
fn test_parse_col_value_text() {
    // hello len 5 * 2 + 13 = 23 -> serial type 23
    // should parse only hello, ignore hi
    let (value, size) = ColumnValue::parse(23, b"hellohi").unwrap();
    assert_eq!(size, 5);
    assert_eq!(value, ColumnValue::Text("hello".to_owned()));
}
