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

## Example: initialize only, two wallets, two vaults (2026-10-20)

Seeds are `["state", user]`, so one wallet maps to exactly one `vault_state`
(and one `vault`, derived from it). To see another vault you need another
wallet. Only `initialize` was run here: no deposit, no withdraw.

```bash
# wallet A already has a vault, so the client refuses before sending anything
ASDF_RUST_VERSION=system cargo run -- --keypair ~/workspace/solana-ai/sol/devnetA initialize
# Error: already initialized — vault_state exists at HSdoaEQ8…

# wallet B: same program, different user → different addresses
ASDF_RUST_VERSION=system cargo run -- --keypair ~/workspace/solana-ai/sol/devnetB initialize
# initialized. tx: 4UdJZktJ3azom9g2imjHBbD3y54DgfJcfvSPkvfVoanusRa37zFWH4PrdNvmG6uJ4DAb6eZUeZVBBLD2ostDExhM
```

| | wallet A `DoLVFC…` | wallet B `CKyoBE…` |
|---|---|---|
| `vault_state` | `HSdoaEQ8DVgVXd9es4rzEq5rFqNquHabHmmcdT3oAQGr` | `7eSK9k7bqjxvA6x2FFLPZs4KQJynkE1ifsWxvedTzgD8` |
| `vault` | `58fRZWudVkQvjJfgCFLxhUwauYmV3pQLNEU2hjokji2R` | `9x5T4eF4VH6LWVZPF3MrrkvYRNnxKeXnyL97WyWHg2UA` |
| `state_bump` / `vault_bump` | 253 / 254 | 255 / 251 |
| vault balance | 0.1 SOL (deposited) | **no account yet** (`AccountNotFound`) |

What wallet B's state shows after `initialize`:

- `vault_state` exists: owner is the vault program, 10 bytes `e4c452a5 62d2eb98 fb ff`
  (discriminator, then `vault_bump` = 0xfb = 251 and `state_bump` = 0xff = 255),
  holding 701,040 lamports of rent.
- `vault` is only an address. `initialize` checks its seeds but never creates
  it, and the ledger has no entry for it until the first deposit moves
  lamports in.
- Wallet B paid 0.00070604 SOL in total: 701,040 lamports of rent plus a
  5,000-lamport fee.
- The bumps differ from wallet A's because each bump is the first off-curve
  result of a deterministic search from 255 downward, and that search runs on
  different inputs.
