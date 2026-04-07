use std::fmt;

#[macro_export]
macro_rules! log {
    ($self:expr, $($arg:tt)*) => {
        $self.log(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_error {
    ($self:expr, $($arg:tt)*) => {
        $self.log_error(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_debug{
    ($self:expr, $($arg:tt)*) => {
        $self.log_debug(format_args!($($arg)*))
    };
}

pub trait Log {
    fn log_prefix(&self) -> String;

    fn log(&self, args: fmt::Arguments<'_>) {
        let prefix = self.log_prefix();
        let message = format!("{}", args);
        tracing::info!("[{}] {}", prefix, message);
    }

    fn log_error(&self, args: fmt::Arguments<'_>) {
        let prefix = self.log_prefix();
        let message = format!("{}", args);
        tracing::error!("[{}] {}", prefix, message);
    }

    fn log_debug(&self, args: fmt::Arguments<'_>) {
        let prefix = self.log_prefix();
        let message = format!("{}", args);
        tracing::debug!("[{}] {}", prefix, message);
    }
}
