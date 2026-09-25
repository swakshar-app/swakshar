//! Values the portal hard-codes on its side of the protocol.

/// Ports the portal tries, in this order, when it looks for a signer.
pub const SIGNER_PORTS: [u16; 5] = [1585, 2095, 2568, 2868, 4587];

/// Greeting `version` the live portal accepted in August 2026. The portal
/// compares it with its own `LATEST_EM_VERSION`, so it is configurable.
pub const DEFAULT_GREETING_VERSION: &str = "2.8";

/// Greeting `ID` the portal requires (`EM_ENTY_ID` in its JavaScript).
pub const SIGNER_ID: &str = "gstnInfy";

/// Largest request frame accepted, in bytes. Real requests are under 1 KiB.
pub const MAX_REQUEST_BYTES: usize = 16 * 1024;

/// The only `signtype` seen in portal flows, and the only one supported.
pub const SUPPORTED_SIGNTYPE: &str = "1";
