#[macro_export]
/// Implements `id()` and `id_str()` getters for each enum variant, returning its numeric ID and stringified ID.
macro_rules! id_str_enum {
    ($enum_name:ident { $($variant:ident => $id:expr),* $(,)? }) => {
        impl $enum_name {
            pub const fn id(&self) -> u64 {
                match self {
                    $(Self::$variant => $id,)*
                }
            }

            pub const fn id_str(&self) -> &'static str {
                match self {
                    $(Self::$variant => stringify!($id),)*
                }
            }
        }
    };
}
