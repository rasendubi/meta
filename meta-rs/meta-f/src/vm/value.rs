//! meta-f values.
//!
//! meta-f values are stored as tagged pointers.
use std::fmt::Debug;

use num_enum::{IntoPrimitive, UnsafeFromPrimitive};

#[derive(Debug, Eq, PartialEq, Copy, Clone, IntoPrimitive, UnsafeFromPrimitive)]
#[repr(u64)]
pub enum ValueTag {
    Pointer = 0b000,
    Number = 0b001,
    Constructor = 0b010,
    FunctionTag = 0b011,
    StringTag = 0b100,
    MovedOut = 0b101,
    SizeTag = 0b110,
    Invalid = 0b111,
}

#[derive(Eq, PartialEq, Copy, Clone, Hash)]
pub struct Value(u64);
impl Value {
    pub(crate) fn from_repr(repr: u64) -> Self {
        Self(repr)
    }
    pub(crate) fn to_repr(self) -> u64 {
        self.0
    }

    pub fn tag(self) -> ValueTag {
        unsafe { ValueTag::from_unchecked(self.0 & 0b111) }
    }

    pub fn from_ptr(ptr: *mut Value) -> Self {
        Self(ptr as u64)
    }
    pub fn as_ptr(self) -> *mut Value {
        assert_eq!(self.tag(), ValueTag::Pointer);
        self.0 as *mut Value
    }

    pub fn number(num: i32) -> Self {
        Self((num as u64) << 32 | u64::from(ValueTag::Number))
    }
    pub fn as_number(self) -> i32 {
        assert_eq!(self.tag(), ValueTag::Number);
        (self.0 >> 32) as i32
    }

    pub(crate) fn size_tag(size: usize) -> Self {
        Self((size as u64) << 16 | u64::from(ValueTag::SizeTag))
    }
    pub(crate) fn as_size_tag(self) -> usize {
        assert_eq!(self.tag(), ValueTag::SizeTag);
        (self.0 >> 16) as usize
    }

    pub fn invalid(meta: i32) -> Self {
        Self((meta as u64) << 32 | u64::from(ValueTag::Invalid))
    }
    pub fn as_invalid(self) -> i32 {
        assert_eq!(self.tag(), ValueTag::Invalid);
        (self.0 >> 32) as i32
    }

    pub fn constructor(type_tag: u64, constructor: u16) -> Self {
        Self(type_tag << 16 | (constructor << 3) as u64 | u64::from(ValueTag::Constructor))
    }
    pub fn as_constructor(self) -> (u64, u16) {
        assert_eq!(self.tag(), ValueTag::Constructor);
        let type_tag = self.0 >> 16;
        let constructor = (self.0 as u16) >> 3;
        (type_tag, constructor)
    }

    pub fn function_tag(n_args: u8) -> Self {
        Self((n_args as u64) << 8 | u64::from(ValueTag::FunctionTag))
    }
    pub fn as_function_tag(self) -> u8 {
        assert_eq!(self.tag(), ValueTag::FunctionTag);
        (self.0 >> 8) as u8
    }

    pub fn string_tag(size: usize) -> Self {
        Self((size as u64) << 32 | u64::from(ValueTag::StringTag))
    }
    pub fn as_string_tag(self) -> usize {
        assert_eq!(self.tag(), ValueTag::StringTag);
        (self.0 >> 32) as usize
    }

    pub fn moved_out(ptr: *mut Value) -> Self {
        let ptr = ptr as usize as u64;
        Self(ptr | u64::from(ValueTag::MovedOut))
    }
    pub fn as_moved_out(self) -> *mut Value {
        assert_eq!(self.tag(), ValueTag::MovedOut);
        (self.0 & !0b111) as usize as *mut Value
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.tag() {
            ValueTag::Pointer => f.debug_tuple("Pointer").field(&self.as_ptr()).finish(),
            ValueTag::Number => f.debug_tuple("Number").field(&self.as_number()).finish(),
            ValueTag::Constructor => {
                let (type_tag, constructor) = self.as_constructor();
                f.debug_struct("Constructor")
                    .field("type_tag", &type_tag)
                    .field("constructor", &constructor)
                    .finish()
            }
            ValueTag::FunctionTag => f
                .debug_tuple("FunctionTag")
                .field(&self.as_function_tag())
                .finish(),
            ValueTag::StringTag => f
                .debug_tuple("StringTag")
                .field(&self.as_string_tag())
                .finish(),
            ValueTag::MovedOut => f
                .debug_tuple("MovedOut")
                .field(&self.as_moved_out())
                .finish(),
            ValueTag::SizeTag => f.debug_tuple("SizeTag").field(&self.as_size_tag()).finish(),
            ValueTag::Invalid => f.debug_tuple("Invalid").field(&self.as_invalid()).finish(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_align_u64() {
        assert_eq!(std::mem::align_of::<Value>(), std::mem::align_of::<u64>());
    }

    #[test]
    fn value_align_f64() {
        assert_eq!(std::mem::align_of::<Value>(), std::mem::align_of::<f64>());
    }

    #[test]
    fn value_align_ptr() {
        assert_eq!(
            std::mem::align_of::<Value>(),
            std::mem::align_of::<*mut u64>()
        );
    }
}
