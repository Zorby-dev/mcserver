use std::marker::PhantomData;

pub struct Var<T>(PhantomData<T>);
pub struct UUID;

#[derive(Debug)]
pub struct Identifier {
    pub namespace: String,
    pub value: String
}

impl ToString for Identifier {
    fn to_string(&self) -> String {
        format!("{}:{}", self.namespace, self.value)
    }
}

pub mod consts {
    pub const VAR_DATA_BITS    : i32 = 0b0111_1111;
    pub const VAR_CONTINUE_BIT : i32 = 0b1000_0000;
}