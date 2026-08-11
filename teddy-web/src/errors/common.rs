//! Common HTTP error handling utilities

pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{e}\n")?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{cause}")?;
        current = cause.source();
    }
    Ok(())
}

#[macro_export]
macro_rules! define_route_error {
    (
        $error_name:ident {
            $(
                $variant:ident => ($status:expr, $message:expr)
            ),* $(,)?
        }
    ) => {
        #[derive(thiserror::Error)]
        pub enum $error_name {
            $(
                #[error($message)]
                $variant,
            )*
            #[error(transparent)]
            UnexpectedError(#[from] anyhow::Error),
        }

        impl std::fmt::Debug for $error_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                $crate::errors::common::error_chain_fmt(self, f)
            }
        }

        impl actix_web::ResponseError for $error_name {
            fn status_code(&self) -> reqwest::StatusCode {
                match self {
                    $(
                        $error_name::$variant => $status,
                    )*
                    $error_name::UnexpectedError(_) => reqwest::StatusCode::INTERNAL_SERVER_ERROR,
                }
            }
        }
    };
}