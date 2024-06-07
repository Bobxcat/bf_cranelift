use std::{
    io::{Read, Write},
    sync::Arc,
};

use byte_chan_active::{ByteRx, ByteTx};

pub trait BfType: Sized {
    fn deserialize(rx: &mut ByteRx) -> Option<Self>;
    fn serialize(&self, tx: &mut ByteTx);
}

macro_rules! impl_bftype_integer {
    ($t:ty) => {
        impl BfType for $t {
            fn deserialize(rx: &mut ByteRx) -> Option<Self> {
                let mut b = [0u8; (Self::BITS / 8) as usize];
                rx.read_exact(&mut b).ok()?;
                Some(Self::from_le_bytes(b))
            }

            fn serialize(&self, tx: &mut ByteTx) {
                let b = self.to_le_bytes();
                tx.write_all(&b).unwrap();
            }
        }
    };
    ($t:ty, $($rhs:ty),+) => {
        impl_bftype_integer!($t);
        impl_bftype_integer!($($rhs),+);
    }
}

impl_bftype_integer!(u8, u16, u32, u64, u128);
impl_bftype_integer!(i8, i16, i32, i64, i128);

#[derive(Debug, Clone)]
pub struct Slice<T> {
    sl: Arc<[T]>,
}

impl<T> Slice<T> {
    pub fn new(shared: Arc<[T]>) -> Self {
        Self { sl: shared }
    }
    pub fn new_vec(v: Vec<T>) -> Self {
        Self::new(Arc::from(v))
    }
    pub fn into_inner(self) -> Arc<[T]> {
        self.sl
    }
    pub fn len(&self) -> u16 {
        self.sl.len().try_into().unwrap()
    }
}

impl<T: BfType> BfType for Slice<T> {
    fn deserialize(rx: &mut ByteRx) -> Option<Self> {
        let len = u16::deserialize(rx)?;
        let mut v = Vec::with_capacity(len as usize);

        for _ in 0..len {
            v.push(T::deserialize(rx)?);
        }

        Some(Self::new_vec(v))
    }

    fn serialize(&self, tx: &mut ByteTx) {
        self.len().serialize(tx);
        for elem in &self.sl[..] {
            elem.serialize(tx);
        }
    }
}
