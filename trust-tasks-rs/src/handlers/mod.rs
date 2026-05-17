//! Reference [`TransportHandler`](crate::TransportHandler) implementations.
//!
//! These are not full transport bindings — they exist to make the trait
//! contract testable end-to-end without pulling in HTTP or DIDComm
//! dependencies. A real binding (REST + mTLS, DIDComm, TSP, ...) lives in a
//! separate crate and implements the trait the same way.

mod in_memory;
mod noop;

pub use in_memory::InMemoryHandler;
pub use noop::NoopHandler;
