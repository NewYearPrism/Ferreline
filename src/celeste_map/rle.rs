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
        use serde::ser::SerializeSeq;
        let (pairs, _) = self.0.as_chunks();
        let mut seq = serializer.serialize_seq(Some(pairs.len()))?;
        for &[count, code] in pairs {
            seq.serialize_element(&(count, char::from(code)))?;
        }
        seq.end()
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
                formatter.write_str("a sequence of pairs of a count number and a char")
            }

            fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
            where
                S: serde::de::SeqAccess<'de>,
            {
                let mut buf = Vec::new_in(Default::default());
                if let Some(count) = seq.size_hint() {
                    buf.reserve_exact(count);
                }
                while let Some((count, code)) = seq.next_element::<(u8, char)>()? {
                    buf.push(count);
                    buf.push(code as _);
                }
                Ok(Rle(buf))
            }
        }
        deserializer.deserialize_seq(_Visitor(Default::default()))
    }
}
