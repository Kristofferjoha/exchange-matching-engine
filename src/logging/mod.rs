
pub mod log_methods;
pub mod logger_trait;
pub mod types;

pub use logger_trait::SimLogger;
pub use types::LoggingMode;

use log_methods::{
    AsyncClosureLogger, AsyncEnumLogger, AsyncStringLogger, BufferedFileWriteLogger,
    NaiveFileWriteLogger, NoOpLogger, PrintlnLogger, TracingLogger
};
use std::path::Path;

pub fn create_logger(mode: LoggingMode) -> Box<dyn SimLogger> {

    const OUTPUT_DIR: &str = "output_logs";

    match mode {
        LoggingMode::Baseline => Box::new(NoOpLogger),
        LoggingMode::Naive => Box::new(PrintlnLogger),
        LoggingMode::NaiveFileWrite => {
            let path = Path::new(OUTPUT_DIR).join("naive_output.log");
            Box::new(NaiveFileWriteLogger::new(path.to_str().unwrap()))
        }
        LoggingMode::BufferedFileWrite => {
            let path = Path::new(OUTPUT_DIR).join("buffered_output.log");
            Box::new(BufferedFileWriteLogger::new(path.to_str().unwrap()))
        }
        LoggingMode::AsyncString => {
            let path = Path::new(OUTPUT_DIR).join("async_string_output.log");
            Box::new(AsyncStringLogger::new(path.to_str().unwrap()))
        }
        LoggingMode::AsyncClosure => {
            let path = Path::new(OUTPUT_DIR).join("async_closure_output.log");
            Box::new(AsyncClosureLogger::new(path.to_str().unwrap()))
        }
        LoggingMode::AsyncEnum => {
            let path = Path::new(OUTPUT_DIR).join("async_enum_output.log");
            Box::new(AsyncEnumLogger::new(path.to_str().unwrap()))
        }

        LoggingMode::TracingFile => {
            let log_file = Path::new(OUTPUT_DIR).join("tracing_output.log");
            let file_appender = tracing_appender::rolling::never("", log_file);
            let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

            let subscriber = tracing_subscriber::fmt()
                .with_writer(non_blocking)
                .with_ansi(false)
                .without_time()
                .with_target(false)
                .with_level(false)
                .compact()
                .finish();
            
            tracing::subscriber::set_global_default(subscriber)
                .expect("Unable to set global tracing subscriber");

            Box::new(TracingLogger::new(Some(guard)))
        }

        LoggingMode::TracingConsole => {
            let subscriber = tracing_subscriber::fmt()
                .with_writer(std::io::stdout)
                .without_time()
                .with_target(false)
                .with_level(false)
                .compact()
                .finish();
            
            tracing::subscriber::set_global_default(subscriber)
                .expect("Unable to set global tracing subscriber");
            
            Box::new(TracingLogger::new(None))
        }
    }
}