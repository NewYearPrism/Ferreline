use allocator_api2::{
    alloc::{
        Allocator,
        Global,
    },
    vec::Vec,
};

#[derive(Debug, Clone)]
pub struct Rle<A: Allocator = Global>(Vec<u8, A>);

impl<A: Allocator> Rle<A> {
    pub fn new(s: Vec<u8, A>) -> Option<Self> {
        s.len().is_multiple_of(2).then(|| Self(s))
    }
}

#[cfg(feature = "serde")]
impl<A: Allocator> serde::Serialize for Rle<A> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut buf = String::new();
        for &[count, code] in self.0.as_chunks().0 {
            buf.push_str(&format!("{:02X}", count));
            buf.push(code as _);
        }
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("rle", &buf)?;
        map.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, A: Allocator + Default> serde::Deserialize<'de> for Rle<A> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct _Visitor<A>(core::marker::PhantomData<A>);
        impl<'de, A: Allocator + Default> serde::de::Visitor<'de> for _Visitor<A> {
            type Value = Rle<A>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a sequence of pairs of count number and char")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                use serde::de::Error;
                let _: &str = map
                    .next_key()?
                    .filter(|&k| k == "rle")
                    .ok_or(Error::missing_field("rle"))?;
                let s: String = map.next_value()?;
                let mut buf = Vec::new_in(Default::default());
                for &[high, low, code] in s.as_bytes().as_chunks().0 {
                    let a: u8 = str::from_utf8(&[high, low])
                        .map(|a| u8::from_str_radix(a, 16))
                        .map_err(M::Error::custom)?
                        .map_err(M::Error::custom)?;
                    buf.push(a);
                    buf.push(code as _);
                }
                Ok(Rle(buf))
            }
        }
        deserializer.deserialize_map(_Visitor(Default::default()))
    }
}
