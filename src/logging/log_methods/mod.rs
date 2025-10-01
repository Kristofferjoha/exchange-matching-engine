pub mod no_logging;
pub mod println;
pub mod naive_file_write;
pub mod buffered_file;
pub mod async_string;
pub mod async_closure;
pub mod async_enum;
pub mod tracing_logger;

pub use async_closure::AsyncClosureLogger;
pub use async_enum::AsyncEnumLogger;
pub use async_string::AsyncStringLogger;
pub use buffered_file::BufferedFileWriteLogger;
pub use naive_file_write::NaiveFileWriteLogger;
pub use no_logging::NoOpLogger;
pub use println::PrintlnLogger;
pub use tracing_logger::TracingLogger;