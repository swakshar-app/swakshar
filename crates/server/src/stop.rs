//! Stopping the server gently. The port closes at once, a request still
//! waiting is answered `signing canceled`, idle pages get a close frame, and
//! whatever has not finished within the drain time is cut off.

use tokio::sync::watch;

/// The owner's side: tells the server to stop.
#[derive(Debug)]
pub struct Stopper(watch::Sender<bool>);

/// The server's side: resolves once a stop is asked for, or once the
/// [`Stopper`] is dropped.
#[derive(Debug, Clone)]
pub struct StopSignal(watch::Receiver<bool>);

/// A connected stopper and signal.
#[must_use]
pub fn stop_pair() -> (Stopper, StopSignal) {
    let (sender, receiver) = watch::channel(false);
    (Stopper(sender), StopSignal(receiver))
}

impl Stopper {
    /// Asks the server to stop. Returns at once; the server drains on its
    /// own.
    pub fn stop(&self) {
        self.0.send_replace(true);
    }
}

impl StopSignal {
    /// Resolves when the server should stop.
    pub(crate) async fn stopped(&mut self) {
        let _ = self.0.wait_for(|stop| *stop).await;
    }
}
