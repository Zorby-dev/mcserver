#[macro_export]
macro_rules! switch {
    ($expression: expr; $ok: literal $(,$arg: expr)*; $err: literal;) => {
        match $expression {
            Ok(value) => {
                paris::info!($ok, $($arg),*);
                value
            }
            Err(error) => {
                paris::error!("{}\n{}", $err, error);
                panic!();
            }
        }
    };
}