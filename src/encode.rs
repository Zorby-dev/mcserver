use std::io::Write;

use crate::types::VarI32;

pub trait Encode {
    fn encode(&self, writer: &mut impl Write);
}

impl Encode for String {
    fn encode(&self, writer: &mut impl Write) {
        VarI32
    }
}