// Imports
use anyhow::Context;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;
use thiserror::Error;
use tracing::error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum PeriodicTaskMsg {
    ChangeTimeout(Duration),
    Skip,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PeriodicTaskResult {
    Continue,
    Quit,
}

#[derive(Debug, Clone)]
pub struct PeriodicTaskHandle {
    tx: std::sync::mpsc::Sender<PeriodicTaskMsg>,
}

impl Drop for PeriodicTaskHandle {
    fn drop(&mut self) {
        if let Err(e) = self.quit() {
            error!("Could not quit periodic task while handle is being dropped, Err: {e:?}");
        }
    }
}

impl PeriodicTaskHandle {
    pub fn new<F>(task: F, timeout: Duration) -> Self
    where
        F: Fn() -> PeriodicTaskResult + Send + 'static,
    {
        let (tx, rx) = std::sync::mpsc::channel::<PeriodicTaskMsg>();
        std::thread::spawn(move || {
            let mut duration = timeout;
            loop {
                match rx.recv_timeout(duration) {
                    Ok(PeriodicTaskMsg::ChangeTimeout(d)) => duration = d,
                    Ok(PeriodicTaskMsg::Skip) => {
                        continue;
                    }
                    Ok(PeriodicTaskMsg::Quit) => {
                        break;
                    }
                    Err(e @ mpsc::RecvTimeoutError::Disconnected) => {
                        error!("Periodic task channel sending half became disconnected, now quitting. Err: {e:?}");
                        break;
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                }
                if task() == PeriodicTaskResult::Quit {
                    break;
                }
            }
        });

        Self { tx }
    }

    pub fn change_timeout(&mut self, timeout: Duration) -> anyhow::Result<()> {
        self.tx
            .send(PeriodicTaskMsg::ChangeTimeout(timeout))
            .context("Sending `ChangeTimeout` message to periodic task failed.")
    }

    pub fn skip(&mut self) -> anyhow::Result<()> {
        self.tx
            .send(PeriodicTaskMsg::Skip)
            .context("Sending `Skip` message to periodic task failed.")
    }

    pub fn quit(&mut self) -> anyhow::Result<()> {
        self.tx
            .send(PeriodicTaskMsg::Quit)
            .context("Sending `Quit` message to periodic task failed.")
    }
}

enum OneOffTaskMsg {
    ChangeAndResetTimeout(Duration),
    ResetTimeout,
    ReplaceTask(Box<dyn Fn() + Send + 'static>),
    Quit,
}

impl std::fmt::Debug for OneOffTaskMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ChangeAndResetTimeout(arg0) => {
                f.debug_tuple("ChangeAndResetTimeout").field(arg0).finish()
            }
            Self::ResetTimeout => write!(f, "ResetTimeout"),
            Self::ReplaceTask(_) => f
                .debug_tuple("ReplaceTask")
                .field(&String::from(".. no debug impl .."))
                .finish(),
            Self::Quit => write!(f, "Quit"),
        }
    }
}

#[derive(Error, Debug)]
pub enum OneOffTaskError {
    #[error("one-time task has already reached the timeout")]
    TimeoutReached,
    #[error("sending message to one time task failed")]
    MsgErr(anyhow::Error),
}

#[derive(Debug, Clone)]
pub struct OneOffTaskHandle {
    msg_tx: std::sync::mpsc::Sender<OneOffTaskMsg>,
    timeout_reached: Arc<AtomicBool>,
}

impl Drop for OneOffTaskHandle {
    fn drop(&mut self) {
        match self.quit() {
            Ok(()) | Err(OneOffTaskError::TimeoutReached) => {}
            Err(e) => {
                error!("Could not quit one off task while handle is being dropped, Err: {e:?}")
            }
        }
    }
}

impl OneOffTaskHandle {
    pub fn new<F>(task: F, timeout: Duration) -> Self
    where
        F: Fn() + Send + 'static,
    {
        let (tx, rx) = std::sync::mpsc::channel::<OneOffTaskMsg>();
        let timeout_reached = Arc::new(AtomicBool::new(false));
        let timeout_reached_c = Arc::clone(&timeout_reached);
        std::thread::spawn(move || {
            let mut timeout = timeout;
            let mut task: Box<dyn Fn() + Send + 'static> = Box::new(task);
            loop {
                match rx.recv_timeout(timeout) {
                    Ok(OneOffTaskMsg::ChangeAndResetTimeout(d)) => {
                        timeout = d;
                        continue;
                    }
                    Ok(OneOffTaskMsg::ResetTimeout) => {
                        continue;
                    }
                    Ok(OneOffTaskMsg::ReplaceTask(t)) => {
                        task = t;
                        continue;
                    }
                    Ok(OneOffTaskMsg::Quit) => {
                        break;
                    }
                    Err(e @ mpsc::RecvTimeoutError::Disconnected) => {
                        error!("One off task channel sending half became disconnected, now quitting. Err: {e:?}");
                        break;
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                }
                timeout_reached_c.store(true, Ordering::Relaxed);
                task();
                break;
            }
        });

        Self {
            msg_tx: tx,
            timeout_reached,
        }
    }

    pub fn change_and_reset_timeout(&mut self, timeout: Duration) -> Result<(), OneOffTaskError> {
        if self.timeout_reached() {
            return Err(OneOffTaskError::TimeoutReached);
        }
        self.msg_tx
            .send(OneOffTaskMsg::ChangeAndResetTimeout(timeout))
            .map_err(|e| {
                OneOffTaskError::MsgErr(anyhow::anyhow!(
                    "Sending `ChangeAndResetTimeout` message to one off task failed, Err: {e:?}"
                ))
            })
    }

    pub fn reset_timeout(&mut self) -> Result<(), OneOffTaskError> {
        if self.timeout_reached() {
            return Err(OneOffTaskError::TimeoutReached);
        }
        self.msg_tx.send(OneOffTaskMsg::ResetTimeout).map_err(|e| {
            OneOffTaskError::MsgErr(anyhow::anyhow!(
                "Sending `ResetTimeout` message to one off task failed, Err: {e:?}"
            ))
        })
    }

    /// Replaces the task, resetting the timeout while doing so.
    pub fn replace_task<F>(&mut self, task: F) -> Result<(), OneOffTaskError>
    where
        F: Fn() + Send + 'static,
    {
        if self.timeout_reached() {
            return Err(OneOffTaskError::TimeoutReached);
        }
        self.msg_tx
            .send(OneOffTaskMsg::ReplaceTask(Box::new(task)))
            .map_err(|e| {
                OneOffTaskError::MsgErr(anyhow::anyhow!(
                    "Sending `ReplaceTask` message to one off task failed, Err: {e:?}"
                ))
            })
    }

    pub fn quit(&mut self) -> Result<(), OneOffTaskError> {
        if self.timeout_reached() {
            return Err(OneOffTaskError::TimeoutReached);
        }
        self.msg_tx.send(OneOffTaskMsg::Quit).map_err(|e| {
            OneOffTaskError::MsgErr(anyhow::anyhow!(
                "Sending `Quit` message to one off task failed, Err: {e:?}"
            ))
        })
    }

    pub fn timeout_reached(&self) -> bool {
        self.timeout_reached.load(Ordering::Relaxed)
    }
}
