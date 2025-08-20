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
                pub async fn $fn_name(
                    data: &std::sync::Arc<tokio::sync::RwLock<serenity::prelude::TypeMap>>,
                    $(
                        $(
                            $placeholder: impl std::fmt::Display,
                        )*
                    )*
                ) -> String {
                    let lock = $crate::shared::get::<$crate::config::MessagesConfig>(data).await;
                    let mut msg = lock.read().await$(.$field)+.to_string();
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
