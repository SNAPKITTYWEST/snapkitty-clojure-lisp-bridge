use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    thread::JoinHandle,
};

use craft_crypto::Dialect;
use soulvm::{BifrostBridge, SoulFunc};

use crate::{
    agent::{AgentMsg, SoulAgent},
    AgentError,
};

struct Lifecycle {
    alive: AtomicBool,
    thread: Mutex<Option<JoinHandle<()>>>,
}

#[derive(Clone)]
pub struct AgentHandle {
    sender: mpsc::Sender<AgentMsg>,
    lifecycle: Arc<Lifecycle>,
}

impl AgentHandle {
    pub fn spawn(soul_id: u64, bridge: Option<BifrostBridge>) -> Result<Self, AgentError> {
        let (sender, inbox) = mpsc::channel();
        let (startup_tx, startup_rx) = mpsc::sync_channel(1);
        let lifecycle = Arc::new(Lifecycle {
            alive: AtomicBool::new(false),
            thread: Mutex::new(None),
        });
        let thread_lifecycle = Arc::clone(&lifecycle);

        let thread = std::thread::Builder::new()
            .name(format!("soul-{soul_id}"))
            .spawn(move || {
                let result = SoulAgent::new(soul_id, bridge, inbox);
                match result {
                    Ok(agent) => {
                        thread_lifecycle.alive.store(true, Ordering::Release);
                        let _ = startup_tx.send(Ok(()));
                        agent.run();
                    }
                    Err(error) => {
                        let _ = startup_tx.send(Err(error));
                    }
                }
                thread_lifecycle.alive.store(false, Ordering::Release);
            })
            .map_err(|error| AgentError::ExecError(error.to_string()))?;

        *lifecycle
            .thread
            .lock()
            .map_err(|_| AgentError::ExecError("agent lifecycle poisoned".into()))? = Some(thread);

        startup_rx.recv().map_err(|_| AgentError::AgentDead)??;

        Ok(Self { sender, lifecycle })
    }

    pub fn load(&self, func: SoulFunc) -> Result<(), AgentError> {
        self.ensure_alive()?;
        let (reply_tx, reply_rx) = mpsc::sync_channel(1);
        self.sender
            .send(AgentMsg::Load {
                func,
                reply: reply_tx,
            })
            .map_err(|_| AgentError::AgentDead)?;
        reply_rx.recv().map_err(|_| AgentError::AgentDead)?
    }

    pub fn load_dialect<D: Dialect>(&self, dialect: &D, source: &str) -> Result<(), AgentError> {
        self.load_named(source, dialect).map(|_| ())
    }

    pub fn load_named<D: Dialect>(&self, source: &str, dialect: &D) -> Result<String, AgentError> {
        let func = dialect
            .compile(source)
            .map_err(|error| AgentError::DialectError(error.to_string()))?;
        let name = func.name.clone();
        self.load(func)?;
        Ok(name)
    }

    pub fn call(&self, name: &str, args: Vec<i64>) -> Result<i64, AgentError> {
        self.ensure_alive()?;
        let (reply_tx, reply_rx) = mpsc::sync_channel(1);
        self.sender
            .send(AgentMsg::Call {
                name: name.to_owned(),
                args,
                reply: reply_tx,
            })
            .map_err(|_| AgentError::AgentDead)?;
        reply_rx.recv().map_err(|_| AgentError::AgentDead)?
    }

    pub fn is_alive(&self) -> bool {
        self.lifecycle.alive.load(Ordering::Acquire)
    }

    pub fn shutdown(self) {
        self.lifecycle.alive.store(false, Ordering::Release);
        let _ = self.sender.send(AgentMsg::Shutdown);
        if let Ok(mut thread) = self.lifecycle.thread.lock() {
            if let Some(thread) = thread.take() {
                let _ = thread.join();
            }
        }
    }

    fn ensure_alive(&self) -> Result<(), AgentError> {
        if self.is_alive() {
            Ok(())
        } else {
            Err(AgentError::AgentDead)
        }
    }
}
