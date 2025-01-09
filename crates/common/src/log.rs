use std::fmt::Display;

pub trait LogError {
    fn log_error(&self) -> &Self;
    fn log_error_msg(&self, msg: &str) -> &Self;
    fn log_error_msg_with<M, F>(&self, f: F) -> &Self
    where
        M: Into<String>,
        F: FnOnce() -> M;
}

impl<T, E> LogError for Result<T, E>
where
    E: Display,
{
    fn log_error(&self) -> &Self {
        if let Err(e) = self {
            log::error!("{e}")
        }
        self
    }
    fn log_error_msg(&self, msg: &str) -> &Self {
        if let Err(e) = self {
            log::error!("{msg}: {e}")
        }
        self
    }
    fn log_error_msg_with<M, F>(&self, f: F) -> &Self
    where
        M: Into<String>,
        F: FnOnce() -> M,
    {
        self.log_error_msg(&f().into());
        self
    }
}
