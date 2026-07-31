use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, SyncSender},
};

use soulvm::{jit::CompiledFunc, BifrostBridge, SoulFunc, SoulRuntime};

use crate::AgentError;

pub enum AgentMsg {
    Load {
        func: SoulFunc,
        reply: SyncSender<Result<(), AgentError>>,
    },
    Call {
        name: String,
        args: Vec<i64>,
        reply: SyncSender<Result<i64, AgentError>>,
    },
    Shutdown,
}

pub(crate) struct SoulAgent {
    runtime: SoulRuntime,
    funcs: HashMap<String, (CompiledFunc, usize)>,
    inbox: Receiver<AgentMsg>,
}

impl SoulAgent {
    pub(crate) fn new(
        soul_id: u64,
        bridge: Option<BifrostBridge>,
        inbox: Receiver<AgentMsg>,
    ) -> Result<Self, AgentError> {
        let mut runtime = SoulRuntime::new(soul_id).map_err(AgentError::CompileError)?;
        if let Some(bridge) = bridge {
            runtime.attach_bifrost(bridge);
        }
        Ok(Self {
            runtime,
            funcs: HashMap::new(),
            inbox,
        })
    }

    pub(crate) fn run(mut self) {
        while let Ok(message) = self.inbox.recv() {
            match message {
                AgentMsg::Shutdown => break,
                AgentMsg::Load { func, reply } => {
                    let name = func.name.clone();
                    let arity = func.params.len();
                    let result = self
                        .runtime
                        .compile(&func)
                        .map(|compiled| {
                            self.funcs.insert(name, (compiled, arity));
                        })
                        .map_err(AgentError::CompileError);
                    let _ = reply.send(result);
                }
                AgentMsg::Call { name, args, reply } => {
                    let _ = reply.send(self.dispatch(&name, &args));
                }
            }
        }
    }

    fn dispatch(&self, name: &str, args: &[i64]) -> Result<i64, AgentError> {
        let (compiled, arity) = self
            .funcs
            .get(name)
            .ok_or_else(|| AgentError::NoSuchFunc(name.to_owned()))?;
        unsafe { exec_compiled(compiled, args, *arity) }
    }
}

unsafe fn exec_compiled(
    func: &CompiledFunc,
    args: &[i64],
    arity: usize,
) -> Result<i64, AgentError> {
    if arity != args.len() {
        return Err(AgentError::ExecError(format!(
            "arity mismatch: expected {arity}, got {}",
            args.len()
        )));
    }

    match arity {
        0 => {
            let function: extern "C" fn() -> i64 = std::mem::transmute(func.code);
            Ok(function())
        }
        1 => {
            let function: extern "C" fn(i64) -> i64 = std::mem::transmute(func.code);
            Ok(function(args[0]))
        }
        2 => {
            let function: extern "C" fn(i64, i64) -> i64 = std::mem::transmute(func.code);
            Ok(function(args[0], args[1]))
        }
        _ => Err(AgentError::ExecError(format!(
            "arity {arity} not yet supported (Sprint 3)"
        ))),
    }
}
