// The bytes inside this struct are reversed (big-endian)
#[derive(Debug)]
pub struct VarI32(pub [u8; 5]);

impl VarI32 {
    pub fn len(&self) -> usize {
        self.0.iter().filter(|i| **i != 0).count()
    }
}

impl From<VarI32> for i32 {
    fn from(value: VarI32) -> Self {
        let mut output: Self = 0;

        for (i, byte) in value.0.iter().enumerate() {
            output |= *byte as i32 & 0x7F << i * 7;
        }

        output
    }
}