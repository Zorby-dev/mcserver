use std::fmt::Debug;

#[derive(Clone)]
pub struct VarI32(pub [u8; VarI32::MAX_LEN], pub u8);

impl VarI32 {
    pub const MAX_LEN: usize = 5;

    pub fn len(&self) -> usize {
        self.1 as usize
    }

    pub fn bytes(&self) -> &[u8] {
        &self.0[0..self.len()]
    }
}

impl From<VarI32> for i32 {
    fn from(value: VarI32) -> Self {
        let mut output: Self = 0;

        for i in 0..value.1 {
            output |= (value.0[i as usize] as i32 & 0x7F) << (i * 7);
        }

        output
    }
}

impl From<i32> for VarI32 {
    fn from(value: i32) -> Self {
        let mut output: [u8; Self::MAX_LEN] = [0; Self::MAX_LEN];
        let mut value = value as u32;

        for i in 0..output.len() {
            if (value & !0x7F) == 0 {
                output[i] = value as u8;
                return Self(output, i as u8 + 1)
            }

            output[i] = (value as u8 & 0x7F) | 0x80;

            value >>= 7;
        };

        unreachable!()
    }
}

impl Debug for VarI32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("VarI32").field(&i32::from(self.clone())).finish()
    }
}

#[derive(Debug)]
pub struct UUID(pub u128);

#[derive(Debug)]
pub struct Identifier {
    pub namespace: String,
    pub value: String
}

impl From<&Identifier> for String {
    fn from(value: &Identifier) -> Self {
        format!("{}:{}", &value.namespace, &value.value)
    }
}

impl From<&str> for Identifier {
    fn from(value: &str) -> Self {
        // FIXME: this mess
        let mut pieces = value.split(':');

        Self {
            namespace: pieces.next().unwrap().to_string(),
            value: pieces.next().unwrap().to_string()
        }
    }
}

#[derive(Debug)]
pub struct Nbt(pub Vec<u8>);

#[derive(Debug)]
pub struct Pos {
    pub x: i32,
    pub y: i16,
    pub z: i32
}