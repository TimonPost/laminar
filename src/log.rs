//! Logging adapter for Laminar
//!
//! This module implements a simple, threaded-logger-friendly logging adapter. Logging adapters are
//! used to attach an arbitrary logger into Laminar.
use std::fmt;
use std::sync::Arc;

/// Logger trait for laminar
///
/// Any user of Laminar can implement this trait to attach their favorite logger to an instance of
/// laminar. The log levels correspond to the same log levels as in the `log` crate.
pub trait LaminarLogger {
    /// Log a trace message
    fn trace(&self, disp: Displayer);
    /// Log a debug message
    fn debug(&self, disp: Displayer);
    /// Log an info message
    fn info(&self, disp: Displayer);
    /// Log a warning message
    fn warn(&self, disp: Displayer);
    /// Log an error message
    fn error(&self, disp: Displayer);
}

// ---

/// Holds a handle to a formatter function while implementing the [fmt::Display] trait.
pub struct Displayer {
    data: Arc<dyn Fn(&mut ::std::fmt::Formatter) -> ::std::fmt::Result + Send + Sync>,
}

impl Displayer {
    pub(crate) fn new(
        delegate: Arc<dyn Fn(&mut ::std::fmt::Formatter) -> ::std::fmt::Result + Send + Sync>,
    ) -> Self {
        Self { data: delegate }
    }
}

impl fmt::Display for Displayer {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        (self.data)(f)
    }
}

// ---

pub(crate) struct DefaultLogger;

impl LaminarLogger for DefaultLogger {
    fn trace(&self, _: Displayer) {}
    fn debug(&self, _: Displayer) {}
    fn info(&self, _: Displayer) {}
    fn warn(&self, _: Displayer) {}
    fn error(&self, _: Displayer) {}
}

// ---

impl fmt::Debug for dyn LaminarLogger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write![f, "LaminarLogger"]
    }
}

// ---

/// Format-friendly form of [log::LaminarLogger::trace]
#[macro_export]
macro_rules! trace {
    ($logger:expr, $($fmt:expr),* $(,)?) => {{
        $logger.trace($crate::log::Displayer::new(::std::sync::Arc::new(move |f: &mut ::std::fmt::Formatter| { write![f, $($fmt),*] }) ));
    }};
}

/// Format-friendly form of [log::LaminarLogger::debug]
#[macro_export]
macro_rules! debug {
    ($logger:expr, $($fmt:expr),* $(,)?) => {{
        $logger.debug($crate::log::Displayer::new(::std::sync::Arc::new(move |f: &mut ::std::fmt::Formatter| { write![f, $($fmt),*] }) ));
    }};
}

/// Format-friendly form of [log::LaminarLogger::info]
#[macro_export]
macro_rules! info {
    ($logger:expr, $($fmt:expr),* $(,)?) => {{
        $logger.info($crate::log::Displayer::new(::std::sync::Arc::new(move |f: &mut ::std::fmt::Formatter| { write![f, $($fmt),*] }) ));
    }};
}

/// Format-friendly form of [log::LaminarLogger::warn]
#[macro_export]
macro_rules! warn {
    ($logger:expr, $($fmt:expr),* $(,)?) => {{
        $logger.warn($crate::log::Displayer::new(::std::sync::Arc::new(move |f: &mut ::std::fmt::Formatter| { write![f, $($fmt),*] }) ));
    }};
}

/// Format-friendly form of [log::LaminarLogger::error]
#[macro_export]
macro_rules! error {
    ($logger:expr, $($fmt:expr),* $(,)?) => {{
        $logger.error($crate::log::Displayer::new(::std::sync::Arc::new(move |f: &mut ::std::fmt::Formatter| { write![f, $($fmt),*] }) ));
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn log_adapter() {
        use crate::log::{Displayer, LaminarLogger};
        use std::{rc::Rc, sync::Arc};

        let mut cfg = Config::default();

        struct MyAdapter {}

        impl LaminarLogger for MyAdapter {
            fn trace(&self, disp: Displayer) {
                println!["trace: {}", disp];
            }
            fn debug(&self, disp: Displayer) {
                println!["debug: {}", disp];
            }
            fn info(&self, disp: Displayer) {
                println!["info: {}", disp];
            }
            fn warn(&self, disp: Displayer) {
                println!["warn: {}", disp];
            }
            fn error(&self, disp: Displayer) {
                println!["An error! {}", disp];
            }
        }

        cfg.logger = Rc::new(MyAdapter {});

        Socket::bind_any_with_config(cfg).unwrap();
    }
}
