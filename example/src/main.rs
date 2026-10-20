//! vault-client — a Rust client for the pre-req vault program.
//!
//! Deliberately hand-rolled: no Anchor client crate, no generated bindings.
//! Everything Anchor hides is visible here: discriminators, PDA derivation,
//! account metas, serialization. Read top to bottom alongside
//! docs/understanding-the-vault.md.

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use sha2::{Digest, Sha256};
use solana_client::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{Keypair, Signer, read_keypair_file},
    transaction::Transaction,
};
use solana_system_interface::program as system_program;
use std::str::FromStr;

// ── On-chain identities ──────────────────────────────────────────────────────
// The program id is just the pubkey of the deploy keypair (declare_id!).
const PROGRAM_ID: &str = "Cdew9Y89YgSVS184JFdo38ouTQbaji8rkzUAUUjdKZGs";
// The Turbin3 registry that withdraw() CPIs into (idls/registration.json).
const REGISTRY_ID: &str = "TRBZyQHB3m68FGeVsqTK39Wm4xejadjVhP5MAZaKWDM";

fn program_id() -> Pubkey {
    Pubkey::from_str(PROGRAM_ID).unwrap()
}

fn registry_id() -> Pubkey {
    Pubkey::from_str(REGISTRY_ID).unwrap()
}

// ── Instruction discriminators ───────────────────────────────────────────────
// Anchor routes instructions by the first 8 bytes of instruction data:
// sha256("global:<instruction_name>")[..8]. The client must compute the same.
fn discriminator(name: &str) -> [u8; 8] {
    let hash = Sha256::digest(format!("global:{name}").as_bytes());
    hash[..8].try_into().unwrap()
}

// ── PDA derivation ───────────────────────────────────────────────────────────
// Pure local hashing — same equations the program checks on-chain.
// These work (and worked) before any account exists.
fn vault_state_pda(user: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"state", user.as_ref()], &program_id()).0
}

fn vault_pda(vault_state: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"vault", vault_state.as_ref()], &program_id()).0
}

// Derived under the REGISTRY's id, not ours — its accounts live in its namespace.
fn application_account_pda(user: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"prereqs", user.as_ref()], &registry_id()).0
}

// ── CLI ──────────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(about = "Client for the pre-req vault program (devnet)")]
struct Cli {
    /// Path to the wallet keypair JSON (payer + user)
    #[arg(short, long, default_value = "~/.config/solana/id.json")]
    keypair: String,

    #[arg(short, long, default_value = "https://api.devnet.solana.com")]
    url: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Show the state of this wallet's vault and registration account
    Status,
    /// Create vault_state and store the PDA bumps
    Initialize,
    /// Move SOL from the wallet into the vault
    Deposit { amount_sol: f64 },
    /// Move SOL back + register the GitHub handle via CPI (one-shot!)
    Withdraw { amount_sol: f64 },
    /// Drain the vault, delete vault_state, refund rent
    Close,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let path = shellexpand_tilde(&cli.keypair);
    let payer = read_keypair_file(&path)
        .map_err(|e| anyhow::anyhow!("cannot read keypair {path}: {e}"))?;
    let rpc = RpcClient::new_with_commitment(cli.url.clone(), CommitmentConfig::confirmed());

    match cli.command {
        Command::Status => status(&rpc, &payer),
        Command::Initialize => initialize(&rpc, &payer),
        Command::Deposit { amount_sol } => deposit(&rpc, &payer, amount_sol),
        Command::Withdraw { amount_sol } => withdraw(&rpc, &payer, amount_sol),
        Command::Close => close(&rpc, &payer),
    }
}

// ── Status: read the ledger, no transaction ──────────────────────────────────

fn status(rpc: &RpcClient, payer: &Keypair) -> Result<()> {
    let user = payer.pubkey();
    let state = vault_state_pda(&user);
    let vault = vault_pda(&state);
    let app = application_account_pda(&user);

    println!("user:                 {user}");
    println!("program:              {}", program_id());
    println!("vault_state PDA:      {state}");
    println!("vault PDA:            {vault}");
    println!("application account:  {app}  (registry program)");
    println!();

    // vault_state: 8B discriminator + vault_bump + state_bump (state.rs)
    match rpc.get_account(&state) {
        Ok(acc) => {
            println!("vault_state: exists ({} bytes data, {} lamports)", acc.data.len(), acc.lamports);
            println!("  vault_bump = {}, state_bump = {}", acc.data[8], acc.data[9]);
        }
        Err(_) => println!("vault_state: does not exist (run `initialize` first)"),
    }

    println!("vault:       {} SOL", rpc.get_balance(&vault)? as f64 / LAMPORTS_PER_SOL as f64);
    println!("wallet:      {} SOL", rpc.get_balance(&user)? as f64 / LAMPORTS_PER_SOL as f64);

    // ApplicationAccount: 8 disc | 32 user | 1 bump | 1 pre_req_ts | 1 pre_req_rs | 4 len | github
    match rpc.get_account(&app) {
        Ok(acc) if acc.data.len() >= 47 => {
            let len = u32::from_le_bytes(acc.data[43..47].try_into().unwrap()) as usize;
            let github = String::from_utf8_lossy(&acc.data[47..47 + len]).to_string();
            println!("registration: exists — github = \"{github}\", pre_req_ts = {}, pre_req_rs = {}",
                acc.data[41] != 0, acc.data[42] != 0);
        }
        _ => println!("registration: none (created by the first `withdraw`)"),
    }
    Ok(())
}

// ── Instructions ─────────────────────────────────────────────────────────────
// Account order must match the #[derive(Accounts)] struct field order exactly —
// the on-chain generated deserializer consumes the list positionally.

fn initialize(rpc: &RpcClient, payer: &Keypair) -> Result<()> {
    let user = payer.pubkey();
    let state = vault_state_pda(&user);
    if rpc.get_account(&state).is_ok() {
        bail!("already initialized — vault_state exists at {state}");
    }

    // initialize.rs: user, vault_state, vault, system_program
    let ix = Instruction::new_with_bytes(
        program_id(),
        &discriminator("initialize"),
        vec![
            AccountMeta::new(user, true),
            AccountMeta::new(state, false),
            AccountMeta::new_readonly(vault_pda(&state), false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
    );
    let sig = send(rpc, payer, vec![ix])?;
    println!("initialized. tx: {sig}");
    Ok(())
}

fn deposit(rpc: &RpcClient, payer: &Keypair, amount_sol: f64) -> Result<()> {
    let user = payer.pubkey();
    let state = vault_state_pda(&user);
    rpc.get_account(&state)
        .context("not initialized — run `initialize` first")?;

    // Instruction data = discriminator + Borsh(args). deposit(amount: u64) → 8B LE.
    let amount = (amount_sol * LAMPORTS_PER_SOL as f64) as u64;
    let mut data = discriminator("deposit").to_vec();
    data.extend_from_slice(&amount.to_le_bytes());

    // deposit.rs: user, vault, vault_state, system_program
    let ix = Instruction::new_with_bytes(
        program_id(),
        &data,
        vec![
            AccountMeta::new(user, true),
            AccountMeta::new(vault_pda(&state), false),
            AccountMeta::new_readonly(state, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
    );
    let sig = send(rpc, payer, vec![ix])?;
    println!("deposited {amount_sol} SOL. tx: {sig}");
    Ok(())
}

fn withdraw(rpc: &RpcClient, payer: &Keypair, amount_sol: f64) -> Result<()> {
    let user = payer.pubkey();
    let state = vault_state_pda(&user);
    rpc.get_account(&state)
        .context("not initialized — run `initialize` and `deposit` first")?;

    // withdraw also CPIs the registry's initialize(github), which CREATES the
    // application account. It can only ever succeed once per wallet — after
    // that, the registry rejects the CPI and the whole withdraw rolls back
    // (atomicity). So guard here instead of burning a fee.
    let app = application_account_pda(&user);
    if rpc.get_account(&app).is_ok() {
        bail!(
            "registration account {app} already exists — the registry's initialize \
             would fail and take the whole withdraw down with it. \
             (This wallet is already registered. Use `close` to finish.)"
        );
    }

    let amount = (amount_sol * LAMPORTS_PER_SOL as f64) as u64;
    let mut data = discriminator("withdraw").to_vec();
    data.extend_from_slice(&amount.to_le_bytes());

    // withdraw.rs: user, vault, vault_state, application_account,
    //              application_program, system_program
    let ix = Instruction::new_with_bytes(
        program_id(),
        &data,
        vec![
            AccountMeta::new(user, true),
            AccountMeta::new(vault_pda(&state), false),
            AccountMeta::new_readonly(state, false),
            AccountMeta::new(app, false),
            AccountMeta::new_readonly(registry_id(), false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
    );
    let sig = send(rpc, payer, vec![ix])?;
    println!("withdrew {amount_sol} SOL and registered. tx: {sig}");
    println!("application account: {app}");
    Ok(())
}

fn close(rpc: &RpcClient, payer: &Keypair) -> Result<()> {
    let user = payer.pubkey();
    let state = vault_state_pda(&user);
    rpc.get_account(&state)
        .context("nothing to close — vault_state does not exist")?;

    // close.rs: user, vault, vault_state (mut — it's being deleted), system_program
    let ix = Instruction::new_with_bytes(
        program_id(),
        &discriminator("close"),
        vec![
            AccountMeta::new(user, true),
            AccountMeta::new(vault_pda(&state), false),
            AccountMeta::new(state, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
    );
    let sig = send(rpc, payer, vec![ix])?;
    println!("closed. vault drained, vault_state deleted, rent refunded. tx: {sig}");
    Ok(())
}

// ── Plumbing ─────────────────────────────────────────────────────────────────

fn send(rpc: &RpcClient, payer: &Keypair, instructions: Vec<Instruction>) -> Result<solana_sdk::signature::Signature> {
    let blockhash = rpc.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&payer.pubkey()),
        &[payer],
        blockhash,
    );
    rpc.send_and_confirm_transaction(&tx).map_err(Into::into)
}

fn shellexpand_tilde(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{home}/{rest}");
        }
    }
    path.to_string()
}
