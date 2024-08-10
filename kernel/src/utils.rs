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

/// Aligned data for the give alignment and data
#[repr(C)]
pub struct AlignedAs<Align, T: ?Sized> {
    pub _align: [Align; 0],
    pub data: T,
}

/// Includes the bytes of the alignment
/// See the https://users.rust-lang.org/t/can-i-conveniently-compile-bytes-into-a-rust-program-with-a-specific-alignment/24049/4
#[macro_export]
macro_rules! include_bytes_align_as {
    ($align_ty:ty, $path:literal) => {{
        use $crate::utils::AlignedAs;

        static ALIGNED: &AlignedAs<$align_ty, [u8]> = &AlignedAs {
            _align: [],
            data: *include_bytes!($path),
        };

        &ALIGNED.data
    }};
}
