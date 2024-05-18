use anyhow::Result;

use super::{
    RespArray, RespBulkError, RespBulkString, RespEncode, RespError, RespInteger, RespMap,
    RespNull, RespNullArray, RespNullBulkString, RespSet, RespSimpleString,
};

// implementation of Redis serialization protocol
/*
    - simple string: "+OK\r\n"
    - error: "-Error message\r\n"
    - bulk error: "!<length>\r\n<error>\r\n"
    - integer: ":[<+|->]<value>\r\n"
    - bulk string: "$<length>\r\n<data>\r\n"
    - null bulk string: "$-1\r\n"
    - array: "*<number-of-elements>\r\n<element-1>...<element-n>"
        - "*2\r\n$3\r\nget\r\n$5\r\nhello\r\n"
    - null array: "*-1\r\n"
    - null: "_\r\n"
    - boolean: "#<t|f>\r\n"
    - double: ",[<+|->]<integral>[.<fractional>][<E|e>[sign]<exponent>]\r\n"
    - big number: "([+|-]<number>\r\n"
    - map: "%<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>"
    - set: "~<number-of-elements>\r\n<element-1>...<element-n>"
*/

const BUF_CAP: usize = 1024;

// - simple string: "+OK\r\n"
impl RespEncode for RespSimpleString {
    fn encode(self) -> Result<Vec<u8>> {
        Ok(format!("+{}\r\n", *self).into())
    }
}

// - error: "-Error message\r\n"
impl RespEncode for RespError {
    fn encode(self) -> Result<Vec<u8>> {
        Ok(format!("-{}\r\n", *self).into())
    }
}

// - bulk error: "!<length>\r\n<error>\r\n"
impl RespEncode for RespBulkError {
    fn encode(self) -> Result<Vec<u8>> {
        Ok(format!(
            "!{}\r\n{}\r\n",
            self.0.len(),
            String::from_utf8(self.0).unwrap()
        )
        .into())
    }
}

// - integer: ":[<+|->]<value>\r\n"
impl RespEncode for RespInteger {
    fn encode(self) -> Result<Vec<u8>> {
        Ok(format!(":{}\r\n", self.0).into())
    }
}

// - bulk string: "$<length>\r\n<data>\r\n"
impl RespEncode for RespBulkString {
    fn encode(self) -> Result<Vec<u8>> {
        Ok(format!(
            "${}\r\n{}\r\n",
            self.0.len(),
            String::from_utf8(self.0).unwrap()
        )
        .into())
    }
}

// - null bulk string: "$-1\r\n"
impl RespEncode for RespNullBulkString {
    fn encode(self) -> Result<Vec<u8>> {
        Ok(b"$-1\r\n".to_vec())
    }
}

// - array: "*<number-of-elements>\r\n<element-1>...<element-n>"
//   - "*2\r\n$3\r\nget\r\n$5\r\nhello\r\n"
const ARRAY_CAP: usize = 4096;
impl RespEncode for RespArray {
    fn encode(self) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(ARRAY_CAP);
        buf.extend_from_slice(&format!("*{}\r\n", self.0.len()).into_bytes());

        for frame in self.0 {
            buf.extend_from_slice(&frame.encode().unwrap());
        }

        Ok(buf)
    }
}

// - null array: "*-1\r\n"
impl RespEncode for RespNullArray {
    fn encode(self) -> Result<Vec<u8>> {
        Ok(b"*-1\r\n".to_vec())
    }
}

// - null: "_\r\n"
impl RespEncode for RespNull {
    fn encode(self) -> Result<Vec<u8>> {
        Ok(b"_\r\n".to_vec())
    }
}

// - boolean: "#<t|f>\r\n"
impl RespEncode for bool {
    fn encode(self) -> Result<Vec<u8>> {
        Ok(format!("#{}\r\n", if self { "t" } else { "f" }).into())
    }
}

// - double: ",[<+|->]<integral>[.<fractional>][<E|e>[sign]<exponent>]\r\n"
impl RespEncode for f64 {
    fn encode(self) -> Result<Vec<u8>> {
        Ok(format!(",{:+e}\r\n", self).into())
    }
}

// - map: "%<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>"
impl RespEncode for RespMap {
    fn encode(self) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(&format!("%{}\r\n", self.0.len()).into_bytes());
        for frame in self.0 {
            buf.extend_from_slice(&frame.0.encode().unwrap());
            buf.extend_from_slice(&frame.1.encode().unwrap());
        }
        Ok(buf)
    }
}

// - set: "~<number-of-elements>\r\n<element-1>...<element-n>"
impl RespEncode for RespSet {
    fn encode(self) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(&format!("~{}\r\n", self.0.len()).into_bytes());
        for frame in self.0 {
            buf.extend_from_slice(&frame.encode().unwrap());
        }
        Ok(buf)
    }
}

#[cfg(test)]
mod tests {

    use anyhow::Ok;

    use crate::resp::RespFrame;

    use super::*;

    #[test]
    fn test_simple_string_encode() -> Result<()> {
        let resp_simple_string: RespFrame = RespSimpleString::new("OK").into();
        let result = resp_simple_string.encode()?;
        assert_eq!(result, b"+OK\r\n");
        Ok(())
    }

    #[test]
    fn test_error_encode() -> Result<()> {
        let resp_error: RespFrame = RespError::new("Error").into();
        let result = resp_error.encode()?;
        assert_eq!(result, b"-Error\r\n");
        Ok(())
    }

    #[test]
    fn test_bulk_error_encode() -> Result<()> {
        let resp_bulk_error: RespFrame = RespBulkError::new("Error").into();
        let result = resp_bulk_error.encode()?;
        assert_eq!(result, b"!5\r\nError\r\n");
        Ok(())
    }

    #[test]
    fn test_integer_encode() -> Result<()> {
        let resp_integer: RespFrame = RespInteger::new(1).into();
        let result = resp_integer.encode()?;
        assert_eq!(result, b":1\r\n");
        Ok(())
    }

    #[test]
    fn test_negnegtive_integer_encode() -> Result<()> {
        let resp_integer: RespFrame = RespInteger::new(-1).into();
        let result = resp_integer.encode()?;
        assert_eq!(result, b":-1\r\n");
        Ok(())
    }

    #[test]
    fn test_bulk_string_encode() -> Result<()> {
        let resp_bulk_string: RespFrame = RespBulkString::new("hello").into();
        let result = resp_bulk_string.encode()?;
        assert_eq!(result, b"$5\r\nhello\r\n");
        Ok(())
    }

    #[test]
    fn test_null_bulk_string_encode() -> Result<()> {
        let resp_null_bulk_string: RespFrame = RespNullBulkString.into();
        let result = resp_null_bulk_string.encode()?;
        assert_eq!(result, b"$-1\r\n");
        Ok(())
    }

    #[test]
    fn test_array_encode() -> Result<()> {
        let frame_vec = vec![
            RespNullBulkString.into(),
            RespBulkString::new("hello").into(),
        ];
        let resp_array = RespArray::new(frame_vec);
        let result = resp_array.encode()?;
        assert_eq!(result, b"*2\r\n$-1\r\n$5\r\nhello\r\n");
        Ok(())
    }

    #[test]
    fn test_null_array_encode() -> Result<()> {
        let resp_null_array: RespFrame = RespNullArray.into();
        let result = resp_null_array.encode()?;
        assert_eq!(result, b"*-1\r\n");
        Ok(())
    }

    #[test]
    fn test_null_encode() -> Result<()> {
        let resp_null: RespFrame = RespNull.into();
        let result = resp_null.encode()?;
        assert_eq!(result, b"_\r\n");
        Ok(())
    }

    #[test]
    fn test_bool_true_encode() -> Result<()> {
        let resp_bool: RespFrame = true.into();
        let result = resp_bool.encode()?;
        assert_eq!(result, b"#t\r\n");
        Ok(())
    }

    #[test]
    fn test_bool_false_encode() -> Result<()> {
        let resp_bool: RespFrame = false.into();
        let result = resp_bool.encode()?;
        assert_eq!(result, b"#f\r\n");
        Ok(())
    }

    #[test]
    fn test_double_encode() -> Result<()> {
        let resp_double: RespFrame = 1.0.into();
        let result = resp_double.encode()?;
        assert_eq!(result, b",+1e0\r\n");
        Ok(())
    }

    #[test]
    fn test_double_negative_encode() -> Result<()> {
        let resp_double: RespFrame = (-1.0).into();
        let result = resp_double.encode()?;
        assert_eq!(result, b",-1e0\r\n");
        Ok(())
    }
}
