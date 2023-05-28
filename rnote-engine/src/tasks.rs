// Imports
use anyhow::Context;
use std::sync::mpsc;
use std::time::Duration;

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
            log::error!("Could not quit periodic task while handle is being dropped, {e:?}");
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
                        log::error!("Channel sending half has become disconnected, now quitting the periodic task. {e:?}");
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
