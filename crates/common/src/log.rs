use std::fmt::Display;

pub trait LogError {
    fn log_error(&self);
    fn log_error_with_msg(&self, msg: &str);
    fn eprint_error_with_msg(&self, msg: &str);
}

impl<T, E> LogError for Result<T, E>
where
    E: Display,
{
    fn log_error(&self) {
        if let Err(e) = self {
            log::error!("{e}")
        }
    }
    fn log_error_with_msg(&self, msg: &str) {
        if let Err(e) = self {
            log::error!("{msg}: {e}")
        }
    }
    fn eprint_error_with_msg(&self, msg: &str) {
        if let Err(e) = self {
            eprintln!("{msg}: {e}")
        }
    }
}
