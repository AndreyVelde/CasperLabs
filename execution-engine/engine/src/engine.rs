use core::marker::PhantomData;
use execution::{exec, Error as ExecutionError};
use parity_wasm::elements::Module;
use storage::{ExecutionEffect, GlobalState, TrackingCopy};
use wasm_prep::process;
use common::key::Key;
use storage::transform::Transform;

pub struct EngineState<T: TrackingCopy, G: GlobalState<T>> {
    // Tracks the "state" of the blockchain (or is an interface to it).
    // I think it should be constrained with a lifetime parameter.
    state: G,
    phantom: PhantomData<T>, //necessary to make the compiler not complain that I don't use T, even though G uses it.
}

#[derive(Debug)]
pub enum Error {
    PreprocessingError { error: String },
    SignatureError { error: String },
    ExecError(ExecutionError),
    StorageError(storage::Error)
}

impl From<storage::Error> for Error {
    fn from(error: storage::Error) -> Self {
        Error::StorageError(error)
    }
}

impl From<ExecutionError> for Error {
    fn from(error: ExecutionError) -> Self {
        Error::ExecError(error)
    }
}

impl<T, G> EngineState<T, G>
where
    T: TrackingCopy,
    G: GlobalState<T>,
{
    pub fn new(state: G) -> EngineState<T, G> {
        EngineState {
            state,
            phantom: PhantomData,
        }
    }
    //TODO run_deploy should perform preprocessing and validation of the deploy.
    //It should validate the signatures, ocaps etc.
    pub fn run_deploy(
        &self,
        module_bytes: &[u8],
        address: [u8; 20],
    ) -> Result<ExecutionEffect, Error> {
        let module = self.preprocess_module(module_bytes)?;
        exec(module, address, &self.state).map_err(|e| e.into())
    }

    pub fn apply_effect(&mut self, key: Key, eff: Transform) -> Result<(), Error> {
        self.state.apply(key, eff).map_err(|err| err.into())
    }

    //TODO: inject gas counter, limit stack size etc
    fn preprocess_module(&self, module_bytes: &[u8]) -> Result<Module, Error> {
        process(module_bytes).map_err(|err_str| Error::PreprocessingError { error: err_str })
    }

    //TODO return proper error
    pub fn validate_signatures(
        &self,
        _deploy: &[u8],
        _signature: &[u8],
        _signature_alg: &str,
    ) -> Result<String, Error> {
        Ok(String::from("OK"))
    }
}