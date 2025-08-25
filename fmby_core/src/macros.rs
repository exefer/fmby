#[macro_export]
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
