mod de;
mod error;
mod ser;

pub use de::{from_reader, from_slice, Deserializer};
pub use error::{Error, Result};
pub use ser::{to_vec, to_writer, Serializer};

#[cfg(test)]
mod test {
    use serde::Serialize;

    use crate::to_vec;

    #[derive(Serialize)]
    struct Test {}

    #[test]
    fn empty_compound_ser() {
        let result = to_vec(&Test {});
        assert_eq!(result.unwrap(), [0xAD, 0x4E, 0x42, 0x54, 0x00, 0x05, 0x80, 0x0a, 0x00, 0x00, 0x00, 0x00]);
    }
}
