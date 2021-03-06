use std::{
    env, fs,
    path::{Path, PathBuf},
    rc::Rc,
};

use lazy_static::lazy_static;

use engine_core::engine_state::{
    execution_result::ExecutionResult,
    genesis::{GenesisAccount, GenesisConfig},
};
use engine_shared::{
    account::Account, additive_map::AdditiveMap, gas::Gas, stored_value::StoredValue,
    transform::Transform,
};
use types::Key;

use crate::internal::{
    DEFAULT_CHAIN_NAME, DEFAULT_GENESIS_TIMESTAMP, DEFAULT_PROTOCOL_VERSION, DEFAULT_WASM_COSTS,
    MINT_INSTALL_CONTRACT, POS_INSTALL_CONTRACT, STANDARD_PAYMENT_INSTALL_CONTRACT,
};

lazy_static! {
    // The location of compiled Wasm files if compiled from the Rust sources within the CasperLabs
    // repo, i.e. 'CasperLabs/execution-engine/target/wasm32-unknown-unknown/release/'.
    static ref RUST_WORKSPACE_WASM_PATH: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("CARGO_MANIFEST_DIR should have parent")
        .join("target")
        .join("wasm32-unknown-unknown")
        .join("release");
    // The location of compiled Wasm files if running from within the 'tests' crate generated by the
    // cargo-casperlabs tool, i.e. 'wasm/'.
    static ref RUST_TOOL_WASM_PATH: PathBuf = env::current_dir()
        .expect("should get current working dir")
        .join("wasm");
    // The location of compiled Wasm files if compiled from the Rust sources within the CasperLabs
    // repo where `CARGO_TARGET_DIR` is set, i.e.
    // '<CARGO_TARGET_DIR>/wasm32-unknown-unknown/release/'.
    static ref MAYBE_CARGO_TARGET_DIR_WASM_PATH: Option<PathBuf> = {
        let maybe_target = std::env::var("CARGO_TARGET_DIR").ok();
        maybe_target.as_ref().map(|path| {
            Path::new(path)
                .join("wasm32-unknown-unknown")
                .join("release")
        })
    };
    // The location of compiled Wasm files if compiled from the Rust sources within the CasperLabs
    // repo, i.e. 'CasperLabs/execution-engine/target/wasm32-unknown-unknown/release/'.
    static ref ASSEMBLY_SCRIPT_WORKSPACE_WASM_PATH: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("CARGO_MANIFEST_DIR should have parent")
        .join("target-as");
    static ref WASM_PATHS: Vec<PathBuf> = get_compiled_wasm_paths();
}

/// Constructs a list of paths that should be considered while looking for a compiled wasm file.
fn get_compiled_wasm_paths() -> Vec<PathBuf> {
    let mut ret = vec![
        // Contracts compiled with typescript are tried first
        #[cfg(feature = "use-as-wasm")]
        ASSEMBLY_SCRIPT_WORKSPACE_WASM_PATH.clone(),
        RUST_WORKSPACE_WASM_PATH.clone(),
        RUST_TOOL_WASM_PATH.clone(),
    ];
    if let Some(cargo_target_dir_wasm_path) = &*MAYBE_CARGO_TARGET_DIR_WASM_PATH {
        ret.push(cargo_target_dir_wasm_path.clone());
    };
    ret
}

/// Reads a given compiled contract file based on path
pub fn read_wasm_file_bytes<T: AsRef<Path>>(contract_file: T) -> Vec<u8> {
    let mut attempted_paths = vec![];

    if contract_file.as_ref().is_relative() {
        // Find first path to a given file found in a list of paths
        for wasm_path in WASM_PATHS.iter() {
            let mut filename = wasm_path.clone();
            filename.push(contract_file.as_ref());
            if let Ok(wasm_bytes) = fs::read(&filename) {
                return wasm_bytes;
            }
            attempted_paths.push(filename);
        }
    }
    // Try just opening in case the arg is a valid path relative to current working dir, or is a
    // valid absolute path.
    if let Ok(wasm_bytes) = fs::read(contract_file.as_ref()) {
        return wasm_bytes;
    }
    attempted_paths.push(contract_file.as_ref().to_owned());

    let mut error_msg =
        "\nFailed to open compiled Wasm file.  Tried the following locations:\n".to_string();
    for attempted_path in attempted_paths {
        error_msg = format!("{}    - {}\n", error_msg, attempted_path.display());
    }

    panic!("{}\n", error_msg);
}

pub fn create_genesis_config(accounts: Vec<GenesisAccount>) -> GenesisConfig {
    let name = DEFAULT_CHAIN_NAME.to_string();
    let timestamp = DEFAULT_GENESIS_TIMESTAMP;
    let mint_installer_bytes = read_wasm_file_bytes(MINT_INSTALL_CONTRACT);
    let proof_of_stake_installer_bytes = read_wasm_file_bytes(POS_INSTALL_CONTRACT);
    let standard_payment_installer_bytes = read_wasm_file_bytes(STANDARD_PAYMENT_INSTALL_CONTRACT);
    let protocol_version = *DEFAULT_PROTOCOL_VERSION;
    let wasm_costs = *DEFAULT_WASM_COSTS;
    GenesisConfig::new(
        name,
        timestamp,
        protocol_version,
        mint_installer_bytes,
        proof_of_stake_installer_bytes,
        standard_payment_installer_bytes,
        accounts,
        wasm_costs,
    )
}

pub fn get_exec_costs<T: AsRef<ExecutionResult>, I: IntoIterator<Item = T>>(
    exec_response: I,
) -> Vec<Gas> {
    exec_response
        .into_iter()
        .map(|res| res.as_ref().cost())
        .collect()
}

pub fn get_success_result(response: &[Rc<ExecutionResult>]) -> &ExecutionResult {
    &*response.get(0).expect("should have a result")
}

pub fn get_precondition_failure(response: &[Rc<ExecutionResult>]) -> String {
    let result = response.get(0).expect("should have a result");
    assert!(
        result.has_precondition_failure(),
        "should be a precondition failure"
    );
    format!("{}", result.error().expect("should have an error"))
}

pub fn get_error_message<T: AsRef<ExecutionResult>, I: IntoIterator<Item = T>>(
    execution_result: I,
) -> String {
    let errors = execution_result
        .into_iter()
        .enumerate()
        .filter_map(|(i, result)| {
            if let ExecutionResult::Failure { error, .. } = result.as_ref() {
                Some(format!("{}: {:?}", i, error))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    errors.join("\n")
}

#[allow(clippy::implicit_hasher)]
pub fn get_account(transforms: &AdditiveMap<Key, Transform>, account: &Key) -> Option<Account> {
    transforms.get(account).and_then(|transform| {
        if let Transform::Write(StoredValue::Account(account)) = transform {
            Some(account.to_owned())
        } else {
            None
        }
    })
}
