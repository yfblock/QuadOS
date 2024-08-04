/// Get the address of the symbol
#[macro_export]
macro_rules! sym_addr {
    ($name:ident) => {{
        extern "C" {
            fn $name();
        }
        $name as usize
    }};
}
