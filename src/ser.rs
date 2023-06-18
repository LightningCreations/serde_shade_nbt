use std::io::Write;

use serde::{ser, Serialize};

use crate::error::{Error, Result};

pub fn to_vec<T: ?Sized + Serialize>(value: &T) -> Result<Vec<u8>> {
    let mut serializer = Serializer::new(Vec::new())?;
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

pub fn to_writer<W: Write, T: ?Sized + Serialize>(writer: W, value: &T) -> Result<()> {
    let mut serializer = Serializer::new(writer)?;
    value.serialize(&mut serializer)
}

enum FieldInfo {
    None,
    Root,
    Named(&'static str),
    InSeq(Option<i32>),
}

impl FieldInfo {
    fn write(&mut self, tag: u8, mut w: impl Write) -> Result<()> {
        let result = match self {
            Self::None => Err(Error::FieldInfoUnset),
            Self::Root => Ok(()),
            Self::InSeq(size) => {
                if let Some(x) = size {
                    w.write_all(&[tag])?;
                    w.write_all(&x.to_le_bytes())?;
                    *size = None;
                }
                Ok(())
            }
            Self::Named(name) => {
                w.write_all(&[tag])?;
                let len = u16::try_from(name.len()).map_err(|_| Error::StrLen(name.len()))?;
                w.write_all(&len.to_le_bytes())?;
                Ok(())
            }
        };
        *self = FieldInfo::None;
        result
    }
}

pub struct Serializer<W: Write> {
    output: W,
    field_info: FieldInfo,
}

impl<W: Write> Serializer<W> {
    pub fn new(mut output: W) -> Result<Self> {
        output.write_all(&[0xad, 0x4e, 0x42, 0x54, 0x00, 0x04, 0x80])?;
        Ok(Self {
            output,
            field_info: FieldInfo::Root,
        })
    }
}

impl<W: Write> ser::Serializer for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    type SerializeMap = Self;
    type SerializeSeq = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;

    fn is_human_readable(&self) -> bool {
        false
    }

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.serialize_u8(v.into())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_u8(v as u8)
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_u16(v as u16)
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_u32(v as u32)
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.serialize_u64(v as u64)
    }

    fn serialize_i128(self, v: i128) -> Result<()> {
        self.serialize_u128(v as u128)
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.field_info.write(0x01, &mut self.output)?;
        self.output.write_all(&[v])?;
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.field_info.write(0x02, &mut self.output)?;
        self.output.write_all(&v.to_le_bytes())?;
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.field_info.write(0x03, &mut self.output)?;
        self.output.write_all(&v.to_le_bytes())?;
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.field_info.write(0x04, &mut self.output)?;
        self.output.write_all(&v.to_le_bytes())?;
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.field_info.write(0x05, &mut self.output)?;
        self.output.write_all(&v.to_le_bytes())?;
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.field_info.write(0x06, &mut self.output)?;
        self.output.write_all(&v.to_le_bytes())?;
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.field_info.write(0x07, &mut self.output)?;
        self.output.write_all(v)?;
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.field_info.write(0x08, &mut self.output)?;
        self.output
            .write_all(&mutf8::utf8_to_mutf8(v.as_bytes())?)?;
        Ok(())
    }

    fn serialize_u128(self, v: u128) -> Result<()> {
        todo!()
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self> {
        todo!()
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<()> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()> {
        todo!()
    }

    fn serialize_none(self) -> Result<()> {
        todo!()
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self> {
        self.field_info.write(0x09, &mut self.output)?;
        let len = len.unwrap_or_else(|| todo!());
        let len = len.try_into().map_err(|_| Error::SeqLen(len))?;
        self.field_info = FieldInfo::InSeq(Some(len));
        Ok(self)
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<()> {
        todo!()
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self> {
        self.field_info.write(0x0a, &mut self.output)?;
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self> {
        todo!()
    }

    fn serialize_tuple(self, len: usize) -> Result<Self> {
        todo!()
    }

    fn serialize_tuple_struct(self, name: &'static str, len: usize) -> Result<Self> {
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self> {
        todo!()
    }

    fn serialize_unit(self) -> Result<()> {
        todo!()
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<()> {
        todo!()
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        todo!()
    }
}

impl<W: Write> ser::SerializeMap for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<()> {
        todo!()
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        todo!()
    }

    fn end(self) -> Result<()> {
        todo!()
    }
}

impl<W: Write> ser::SerializeSeq for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<W: Write> ser::SerializeStruct for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<()> {
        self.field_info = FieldInfo::Named(key);
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output.write_all(&[0])?;
        Ok(())
    }
}

impl<W: Write> ser::SerializeStructVariant for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<()> {
        todo!()
    }

    fn end(self) -> Result<()> {
        todo!()
    }
}

impl<W: Write> ser::SerializeTuple for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        todo!()
    }

    fn end(self) -> Result<()> {
        todo!()
    }
}

impl<W: Write> ser::SerializeTupleStruct for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        todo!()
    }

    fn end(self) -> Result<()> {
        todo!()
    }
}

impl<W: Write> ser::SerializeTupleVariant for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        todo!()
    }

    fn end(self) -> Result<()> {
        todo!()
    }
}
