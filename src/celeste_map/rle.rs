use allocator_api2::{
    alloc::{
        Allocator,
        Global,
    },
    vec::Vec,
};
use ascii::{
    AsciiChar,
    AsciiStr,
    ToAsciiChar,
};
use u4::{
    U4,
    U4x2,
};

#[derive(Debug, Clone)]
pub struct Rle<A: Allocator = Global>(Vec<u8, A>);

impl<A: Allocator> Rle<A> {
    pub fn new(s: Vec<u8, A>) -> Option<Self> {
        s.len().is_multiple_of(2).then(|| Self(s))
    }
}

#[derive(Debug, derive_more::Display, derive_more::Error, derive_more::From)]
pub enum DecodeHexError {
    NotMultipleOf3,
    NotHex,
}

impl<A: Allocator> Rle<A> {
    /// Represent the RLE by converting the length bytes into 2-digit upper hex
    /// and leaving the data bytes unchanged.
    ///
    /// For example (48 == '0' and so on):<br/>
    /// [0x05, 48, 0x3e, 49] => "0503E1"
    pub fn to_hex_string(&self) -> String {
        let mut buffer = String::with_capacity(self.0.len() * 3 / 2);
        let (pairs, _) = self.0.as_chunks();
        for &[count, code] in pairs {
            buffer.push_str(<&AsciiStr>::from(byte_to_upper_hex(count).as_ref()).into());
            buffer.push(char::from(code));
        }
        buffer
    }
}

impl<A: Allocator + Default> Rle<A> {
    pub fn decode_hex(s: impl AsRef<[u8]>) -> Result<Self, DecodeHexError> {
        let s = s.as_ref();
        if !s.len().is_multiple_of(3) {
            Err(DecodeHexError::NotMultipleOf3)?
        }
        let (pairs, _) = s.as_chunks();
        let len_in_pair = pairs.len();
        let mut buf = Vec::with_capacity_in(len_in_pair * 2, Default::default());
        for &[high, low, code] in pairs {
            let high_u4 = high
                .to_ascii_char()
                .ok()
                .and_then(hex_to_u4)
                .ok_or(DecodeHexError::NotHex)?;
            let low_u4 = low
                .to_ascii_char()
                .ok()
                .and_then(hex_to_u4)
                .ok_or(DecodeHexError::NotHex)?;
            let count = U4x2::new(high_u4, low_u4).into();
            buf.push(count);
            buf.push(code);
        }
        Ok(Rle(buf))
    }
}

fn byte_to_upper_hex(a: u8) -> [AsciiChar; 2] {
    let split = U4x2::from_byte(a);
    [split.left(), split.right()].map(u4_to_upper_hex)
}

fn u4_to_upper_hex(code: U4) -> AsciiChar {
    match code {
        U4::Dec00 => AsciiChar::_0,
        U4::Dec01 => AsciiChar::_1,
        U4::Dec02 => AsciiChar::_2,
        U4::Dec03 => AsciiChar::_3,
        U4::Dec04 => AsciiChar::_4,
        U4::Dec05 => AsciiChar::_5,
        U4::Dec06 => AsciiChar::_6,
        U4::Dec07 => AsciiChar::_7,
        U4::Dec08 => AsciiChar::_8,
        U4::Dec09 => AsciiChar::_9,
        U4::Dec10 => AsciiChar::A,
        U4::Dec11 => AsciiChar::B,
        U4::Dec12 => AsciiChar::C,
        U4::Dec13 => AsciiChar::D,
        U4::Dec14 => AsciiChar::E,
        U4::Dec15 => AsciiChar::F,
    }
}

fn hex_to_u4(hex: AsciiChar) -> Option<U4> {
    Some(match hex {
        AsciiChar::_0 => U4::Dec00,
        AsciiChar::_1 => U4::Dec01,
        AsciiChar::_2 => U4::Dec02,
        AsciiChar::_3 => U4::Dec03,
        AsciiChar::_4 => U4::Dec04,
        AsciiChar::_5 => U4::Dec05,
        AsciiChar::_6 => U4::Dec06,
        AsciiChar::_7 => U4::Dec07,
        AsciiChar::_8 => U4::Dec08,
        AsciiChar::_9 => U4::Dec09,
        AsciiChar::A | AsciiChar::a => U4::Dec10,
        AsciiChar::B | AsciiChar::b => U4::Dec11,
        AsciiChar::C | AsciiChar::c => U4::Dec12,
        AsciiChar::D | AsciiChar::d => U4::Dec13,
        AsciiChar::E | AsciiChar::e => U4::Dec14,
        AsciiChar::F | AsciiChar::f => U4::Dec15,
        _ => None?,
    })
}

#[cfg(feature = "serde")]
impl<A: Allocator> serde::Serialize for Rle<A> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_hex_string())
    }
}

#[cfg(feature = "serde")]
impl<'de, A: Allocator + Default> serde::Deserialize<'de> for Rle<A> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct StringVisitor<A>(core::marker::PhantomData<A>);
        impl<'de, A: Allocator + Default> serde::de::Visitor<'de> for StringVisitor<A> {
            type Value = Rle<A>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(
                    "a string with multiple of 3 characters \
                where every 3 characters are two hex numbers and one ASCII character",
                )
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Rle::decode_hex(v).map_err(E::custom)
            }
        }
        deserializer.deserialize_str(StringVisitor(Default::default()))
    }
}
