# vault-client

A hand-rolled Rust CLI client for the pre-req vault program
(`Cdew9Y89YgSVS184JFdo38ouTQbaji8rkzUAUUjdKZGs` on devnet).

No Anchor client crate and no generated bindings on purpose: discriminators are
computed (sha256 of `global:<name>`), PDAs are derived locally, instruction data
is Borsh-encoded by hand, and account metas are listed positionally to match each
`#[derive(Accounts)]` struct field order. Everything the framework hides is
visible here.

## Commands

| command | on-chain effect | touches the Turbin3 registry? |
|---|---|---|
| `status` | reads PDAs + decodes accounts (no transaction) | reads only |
| `initialize` | creates `vault_state`, stores bumps | no |
| `deposit <sol>` | wallet → vault (wallet-signed CPI) | no |
| `withdraw <sol>` | vault → wallet (PDA-signed CPI) **+ registration CPI** | **yes — required by the program's `withdraw`** |
| `close` | drains vault, deletes `vault_state`, refunds rent | no |

`withdraw` cannot avoid the registry: the `Withdraw` accounts struct on-chain
requires `application_account` and `application_program`, and the handler CPIs
the registry's `initialize(github)`. Since registration is one-shot per wallet,
a wallet that's already registered can't withdraw again (the registry would
reject its `initialize` and the whole transaction rolls back) — the client
guards this client-side. Use `close` to recover funds after registration.

## Usage

```bash
ASDF_RUST_VERSION=system cargo run -- status
ASDF_RUST_VERSION=system cargo run -- --keypair ~/workspace/solana-ai/sol/devnetA initialize
ASDF_RUST_VERSION=system cargo run -- --keypair ~/workspace/solana-ai/sol/devnetA deposit 0.25
ASDF_RUST_VERSION=system cargo run -- --keypair ~/workspace/solana-ai/sol/devnetA close
```

Flags: `--keypair <path>` (default `~/.config/solana/id.json`), `--url <rpc>`
(default devnet).

## Verified live (2026-10-20)

`status` against devnet after a full test lifecycle:

```
vault_state: does not exist (run `initialize` first)     ← closed ✔
vault:       0 SOL                                       ← drained ✔
registration: exists — github = "rafael-ariascalles"     ← from the withdraw CPI ✔
```

The Rust derivation of all three PDAs matches the TypeScript test's derivation
byte-for-byte — same seeds, same program ids, same addresses, computed
independently.
