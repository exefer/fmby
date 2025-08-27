#[macro_export]
/// Generates string-returning getter methods for bot messages with optional placeholder substitution.
macro_rules! generate_message_getters {
    (
        $struct_ty:ty,
        $(
            $($field:ident).+ => $fn_name:ident $([$($placeholder:ident),*])*
        ),* $(,)?
    ) => {
        impl $struct_ty {
            $(
                pub fn $fn_name(
                    data: &std::sync::Arc<$crate::structs::Data>,
                    $(
                        $(
                            $placeholder: impl std::fmt::Display,
                        )*
                    )*
                ) -> String {
                    let mut msg = data.config.bot_messages$(.$field)+.to_string();
                    $(
                        $(
                            msg = msg.replace(
                                concat!("{", stringify!($placeholder), "}"),
                                &format!("{}", $placeholder),
                            );
                        )*
                    )*
                    msg
                }
            )*
        }
    };
}

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
