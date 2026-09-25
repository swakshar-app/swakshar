//! Loopback `wss://` server that speaks the GST portal's signer protocol.
//!
//! The server only ever binds `127.0.0.1`, checks `Host` and `Origin` before
//! upgrading a connection, sends the greeting, and hands each sign request to
//! a [`Broker`], which decides (with the user) what to reply.

mod broker;
mod connection;
mod error;
mod http;
mod listener;
mod session;
mod settings;
mod status_page;

pub use broker::{Broker, PortalRequest, ServerEvent};
pub use error::ServerError;
pub use listener::{bind_signer_port, serve};
pub use settings::{ServerSettings, tls_config};
