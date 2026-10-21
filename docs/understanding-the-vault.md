# Understanding the Pre-req Vault

*A deep dive into how this Anchor program works — entry points, PDAs, CPIs — verified against a live devnet deployment.*
*Started 2026-10-19, completed 2026-10-20 with a full on-chain lifecycle run.*

> **Status: deployed and verified on devnet.** Program `Cdew9Y89YgSVS184JFdo38ouTQbaji8rkzUAUUjdKZGs`,
> full lifecycle executed (initialize → deposit → withdraw → close), registration CPI confirmed on-chain.

---

## 0. The on-chain evidence

Everything in this document corresponds to real, live state you can inspect:

### Program & wallet

| what | address | explorer |
|---|---|---|
| **This program** | `Cdew9Y89YgSVS184JFdo38ouTQbaji8rkzUAUUjdKZGs` | [view](https://explorer.solana.com/address/Cdew9Y89YgSVS184JFdo38ouTQbaji8rkzUAUUjdKZGs?cluster=devnet) |
| Program's on-chain IDL | `jEH795LRC7sWGGkimgaSzWAjj3X1vBtRxjBm875HKc5` | [view](https://explorer.solana.com/address/jEH795LRC7sWGGkimgaSzWAjj3X1vBtRxjBm875HKc5?cluster=devnet) |
| User wallet (signer/payer) | `DoLVFCsfpqja5PTSFQHML7Lvu5mZHEon6FzDtDUoRmiN` | [view](https://explorer.solana.com/address/DoLVFCsfpqja5PTSFQHML7Lvu5mZHEon6FzDtDUoRmiN?cluster=devnet) |
| Turbin3 registry program | `TRBZyQHB3m68FGeVsqTK39Wm4xejadjVhP5MAZaKWDM` | [view](https://explorer.solana.com/address/TRBZyQHB3m68FGeVsqTK39Wm4xejadjVhP5MAZaKWDM?cluster=devnet) |

### PDAs derived for this wallet (deterministic — anyone can recompute)

| account | seeds | derived address | state now |
|---|---|---|---|
| `vault_state` | `["state", user]` under this program | `HSdoaEQ8DVgVXd9es4rzEq5rFqNquHabHmmcdT3oAQGr` | **exists again** — deleted by `close`, then re-`initialize`d at the identical address ([view](https://explorer.solana.com/address/HSdoaEQ8DVgVXd9es4rzEq5rFqNquHabHmmcdT3oAQGr?cluster=devnet)) |
| `vault` | `["vault", vault_state]` under this program | `58fRZWudVkQvjJfgCFLxhUwauYmV3pQLNEU2hjokji2R` | **exists again** — re-initialized and holding 0.1 SOL (left open on purpose) |
| `application_account` | `["prereqs", user]` under the **registry** | `2D44dQv5d6RwuH3pb5fFVqsg6W8fVPEwTJLYzmmZ8vmj` | **exists** — created by the CPI in `withdraw` ([view](https://explorer.solana.com/address/2D44dQv5d6RwuH3pb5fFVqsg6W8fVPEwTJLYzmmZ8vmj?cluster=devnet)) |

### Second wallet (devnetB `CKyoBEBwnvkat36UBYmZBkisNrspkoKGjqUT6C2vkavg`) — initialized only, no deposit

| account | derived address | state now |
|---|---|---|
| `vault_state` | `7eSK9k7bqjxvA6x2FFLPZs4KQJynkE1ifsWxvedTzgD8` | **exists** — 10 bytes, bumps 255/251 |
| `vault` | `9x5T4eF4VH6LWVZPF3MrrkvYRNnxKeXnyL97WyWHg2UA` | **no account** — `AccountNotFound`, as designed |

Different wallet → different addresses, even though every PDA comes out of the same program and the same seed recipe. That's the one-vault-per-wallet rule in action.

### The lifecycle transactions (2026-10-20, all confirmed)

| instruction | signature | explorer |
|---|---|---|
| deploy + IDL init | `YNQTmN7T…SYA8MgV` | [view](https://explorer.solana.com/tx/YNQTmN7Th3rpzYmD9wQeMBuFeUZfCei6BvDVPFgVz8v8yT9bUmFgrAc8AJAc1M6Uwtc7gDfLZ7x3DjNDSYA8MgV?cluster=devnet) |
| `initialize` | `iiox3Xjh…J97xd` | [view](https://explorer.solana.com/tx/iiox3Xjh1bN7VzU5bBEVQiuy9b5YZrJUs1qFab5B4qswJc8iPBv8uCHrUoAKtJd4v5eTFtJEsNcJwkpG7BJ97xd?cluster=devnet) |
| `deposit` (1 SOL) | `4Ut7WaBf…ddivhb` | [view](https://explorer.solana.com/tx/4Ut7WaBfNmG5WHaq2iuY1e5MQDQusD7H6YqKESmbq1abL5VqW3BkGhuVRkYHkUgu4wX75mA3KkYrmrTcapddivhb?cluster=devnet) |
| `withdraw` (0.5 SOL) **+ registration CPI** | `3mHuxxQe…brzugc` | [view](https://explorer.solana.com/tx/3mHuxxQeGBXmevbFXFKgdPX39pgCf3gf5KDVAy4xpAwkVokQng8D7eXSqDXyFLZ5UAg97urcrY7zwfLQbwbrzugc?cluster=devnet) |
| `close` | `7GWUgfGU…HxmQq` | [view](https://explorer.solana.com/tx/7GWUgfGU188VDGRb4a9DvBHwV64C2ofRP64gNCdFuU18FA7EARKJvi4V8DD98hLq5hdptTqiSA8wUo2fDkHxmQq?cluster=devnet) |
| re-`initialize` (wallet A) | `2UUDJX9r…JG9ASC` | [view](https://explorer.solana.com/tx/2UUDJX9rp3ocQpYn6CqRw35e7p1AXBEBBAdpWkmwC8ZnvoSiEwVCYUFGseUEdCmWNn1amm64WSjrLBPQhvJG9ASC?cluster=devnet) |
| `deposit` 0.1 SOL (wallet A, left open) | `4w7BvUCs…vHvjW3` | [view](https://explorer.solana.com/tx/4w7BvUCsvFXYmzohHswS6k2yC9mFFfMTMTpmFcMXSxxRVcG2FmgXnTZjBmraaf19a3zqrF5W6cWEZYH3mowHvjW3?cluster=devnet) |
| `initialize` (wallet B, no deposit) | `4UdJZktJ…tDExhM` | [view](https://explorer.solana.com/tx/4UdJZktJ3azom9g2imjHBbD3y54DgfJcfvSPkvfVoanusRa37zFWH4PrdNvmG6uJ4DAb6eZUeZVBBLD2ostDExhM?cluster=devnet) |

### The decoded registration account (fetched from devnet after the test)

```
2D44dQv5d6RwuH3pb5fFVqsg6W8fVPEwTJLYzmmZ8vmj
├── owner:      TRBZyQHB…            ← the registry owns it; only it can write it
├── user:       DoLVFCsf…            ← bound as a PDA seed
├── bump:       255                  ← first derivation attempt hit an off-curve address
├── preReqTs:   false                ← completed via submit_ts (separate step)
├── preReqRs:   false                ← flips on submit_rs of this vault program
└── github:     "rafael-ariascalles" ← the string's final resting place
```

---

## 1. File map — where everything lives

```
pre-req-vault/
├── Anchor.toml                        # toolchain, program id per cluster, provider (devnet + wallet), test scripts
├── idls/registration.json             # INTERFACE of the external Turbin3 registry (input to declare_program!)
├── programs/pre-req-vault/
│   ├── src/
│   │   ├── lib.rs                     # declare_id! + #[program]: the 4 public instruction handlers
│   │   ├── constants.rs               # GITHUB_HANDLE — baked into the binary at compile time
│   │   ├── state.rs                   # VaultState { vault_bump, state_bump } — the account data layout
│   │   ├── error.rs                   # custom error definitions (constraint failures)
│   │   ├── instructions.rs            # module wiring
│   │   └── instructions/
│   │       ├── initialize.rs          # creates vault_state, stores bumps
│   │       ├── deposit.rs             # wallet → vault, wallet-signed CPI
│   │       ├── withdraw.rs            # vault → wallet, PDA-signed CPI  +  registry CPI (the checkpoint)
│   │       └── close.rs               # drains vault, deletes vault_state, refunds rent
│   └── tests/test_initialize.rs       # Rust-side test (LiteSVM), run via cargo test
├── tests/pre-req-vault.ts             # the client: derives PDAs off-chain, builds txs, asserts state
└── target/  (git-ignored, regenerated)
    ├── deploy/pre_req_vault-keypair.json   # THE program identity — losing it = new program id
    ├── deploy/pre_req_vault.so             # the deployed binary (152 KB)
    ├── idl/pre_req_vault.json              # this program's own generated interface
    └── types/                              # TS types consumed by tests/clients
```

## 2. The mental model

1. **Programs are stateless.** They contain logic, not data.
2. **Accounts are the only durable state.** Everything that must survive a transaction lives in an account.
3. **Transactions carry intent, then vanish.** Arguments flow in, get used, and only persist if written into an account.

And the runtime rules that make the design what it is:

- **A program may only write accounts it owns** → every interesting operation across ownership boundaries is a **CPI**: a program asking another program to run one of its instructions.
- **Privileges only narrow in a CPI** → the callee gets at most the signer/writable rights the caller had.
- **`invoke_signed`** → a program proves authority over its own PDAs by supplying seeds; the runtime re-derives the address and confirms it could only come from that program.
- **Transactions are atomic** → if any instruction or nested CPI fails, every effect rolls back.

## 3. Entry point and dispatch

There is no `main()`. `declare_id!` pins the program's on-chain address; `#[program]` generates the runtime `entrypoint`, the dispatcher, and argument decoding. Each `pub fn` becomes an instruction; the first 8 bytes of instruction data are the discriminator `sha256("global:<fn_name>")[..8]` that routes the call:

```
discriminator → handler
sha256("global:initialize")[..8] → initialize(ctx)
sha256("global:deposit")[..8]    → deposit(ctx, amount: u64)
sha256("global:withdraw")[..8]   → withdraw(ctx, amount: u64)
sha256("global:close")[..8]      → close(ctx)
```

Every handler receives `Context<T>` — the general wrapper carrying the invocation: validated `accounts`, `program_id`, discovered PDA `bumps`, and any `remaining_accounts`. `T` is the instruction-specific `#[derive(Accounts)]` struct. **Validation happens before the handler body runs**: signer checks, PDA re-derivation (`seeds`/`bump`), owner and discriminator checks, writability. A failed check rejects the transaction before any business logic executes.

## 4. The accounts

| account | seeds | owner | data | created | dies |
|---|---|---|---|---|---|
| `vault_state` | `["state", user]` | **this program** | 8B discriminator + `vault_bump` + `state_bump` | `initialize` (`init`, payer = user) | `close` (`close = user`, rent refunded) |
| `vault` | `["vault", vault_state]` | **system program** | none — lamports only | first `deposit` (materialized by transfer, no explicit create) | `close` (drained to 0, garbage-collected) |
| `application_account` | `["prereqs", user]` under **registry** | **registry program** | `ApplicationAccount` (see §8) | inside the registry CPI during `withdraw` | persists after this program's accounts are gone |

Design split: **programs own information; the system program owns money.** The vault is a bare lamport bucket (a `SystemAccount` — no data, system-owned) whose address is a PDA of this program, so no private key can exist for it. Anyone can deposit; only the program's logic can release.

Note the address/account distinction proven live in this session: all three PDA addresses above were computable before anything existed (we derived them off-chain with `findProgramAddressSync`), and devnet answered `AccountNotFound` until transactions materialized them.

### Two ways an account appears on the ledger (decoded from the live txs)

The system program's `transfer` checks only the sender (`system_processor.rs: transfer_verified` — `from` must carry no data and hold enough lamports). It never looks at the destination: no signature, no existence check. So a plain transfer **materializes an account** at any address — someone-writable form: owner = system program, 0 bytes data. The 0.1 SOL redeposit into wallet A's vault ran exactly that:

```json
{"type":"transfer","info":{"source":"DoLVFCsfpq…","destination":"58fRZWudVk…","lamports":100000000}}
```

Giving an account **data or a different owner** is different: `allocate` and `assign` both require *the address itself to sign*. A PDA can't have a private key, so the only possible signer is the program supplying the seeds — exactly what `invoke_signed` is for. Wallet B's `initialize` ran the full `createAccount` (fund + allocate + assign in one shot):

```json
{"type":"createAccount","info":{"source":"CKyoBEBw…","newAccount":"7eSK9k7b…",
                                "lamports":701040,"space":10,"owner":"Cdew9Y89…"}}
```

Two consequences worth remembering:

- **`createAccount` refuses a funded address** — it bails with `AccountAlreadyInUse` if the target already holds lamports. That's why Anchor's `init` has a fallback: if someone already sent SOL to the PDA, it tops the rent up and calls `allocate` + `assign` instead (still seed-signed). Without that, anyone could grief your `initialize` by sending dust to your future `vault_state` first.
- **The rent floor is enforced at transaction commit, not by the transfer handler.** As of 2026-10-20 on devnet: 650,240 lamports for an empty account, 701,040 for a 10-byte one (matches `vault_state` exactly). Practical rules for this vault: a first deposit below 650,240 lamports fails; a partial withdraw that leaves the vault between 1 and 650,239 lamports fails; `close` draining it to exactly 0 deletes the row entirely.

### Why `UncheckedAccount` for `application_account`

The account *doesn't exist yet* when validation runs — the CPI creates it. Its address is still fully verified (`seeds` + `seeds::program = application_program.key()` derives under `TRBZyQHB…`), so only its data is unchecked. The `/// CHECK:` comment documents this.

### The attribute vocabulary

| attribute | meaning |
|---|---|
| `mut` | writable — the transaction will write this account (runtime locking) |
| `seeds` + `bump` | verify the address re-derives from these inputs (PDA identity check) |
| `seeds::program = …` | derive under a *different* program's ID (cross-program PDA) |
| `init, payer, space` | create the account now via CPI to the system program |
| `close = user` | delete at end of instruction, refund rent |
| `constraint = …` | custom business rule on data |

## 5. Instruction × account matrix

| instruction | purpose | accounts touched | state change |
|---|---|---|---|
| `initialize` | bootstrap a user's vault | creates `vault_state` (init, payer=user, seeds check); verifies `vault` address | `vault_state` row appears; both bumps stored |
| `deposit(amount)` | fund the vault | `user` (signer), `vault` (mut), `vault_state`, `system_program` | CPI `transfer` user→vault; **vault row materializes** with `amount` lamports |
| `withdraw(amount)` | refund + register | same three + `application_account` (PDA under registry) + `application_program` | CPI `transfer` vault→user **signed with vault seeds**; then CPI `initialize(github)` into the registry |
| `close` | teardown | `user`, `vault`, `vault_state` | drains vault to 0; deletes `vault_state`; refunds all rent to user |

## 6. State changes over time — as executed on devnet

```
deploy       → program binary (and its IDL) land at Cdew9Y89…      TX YNQTmN7T…
initialize   → vault_state row created at HSdoaEQ8…                TX iiox3Xjh…
               vault address 58fRZWud… still has NO row
deposit 1SOL → vault row materializes, balance = 1.0 SOL           TX 4Ut7WaBf…
withdraw .5  → vault balance = 0.5 SOL; registry creates           TX 3mHuxxQe…
               2D44dQv5… with github "rafael-ariascalles"
close        → vault → user (0.5 SOL); vault_state deleted;        TX 7GWUgfGU…
               rent refunded. Verified after close: both null.
(again, wallet A)
initialize   → vault_state row back at HSdoaEQ8… — SAME address     TX 2UUDJX9r…
               (seeds are deterministic: the deleted row's address
               is exactly the one that re-derives)
deposit 0.1  → vault 58fRZWud… back with 0.1 SOL (left open)       TX 4w7BvUCs…
(wallet B, initialize only — a second user: a second vault)
initialize   → vault_state 7eSK9k7b…; vault address 9x5T4eF4…      TX 4UdJZktJ…
               still AccountNotFound (never received lamports)
```

After `close`, this program's account rows are gone — the ledger is as if the vault never happened — **except** the `application_account` at the registry, which persists because the registry owns it and its lifecycle belongs to the course, not to this vault.

## 7. The cross-program invocation (the checkpoint)

The registry (`q3_pre_reqs_rs` on devnet) is the course's per-wallet registry. Its interface is consumed through `declare_program!(registration)` in `withdraw.rs:8`, which compiles `idls/registration.json` into a typed module — no runtime link, just locally generated types: `registration::ID`, `registration::program::Q3PreReqsRs`, `registration::cpi::initialize(ctx, github)`, `registration::accounts::ApplicationAccount`, and the registry's error enum.

The implementation (as deployed):

```rust
let registration_accounts = registration::cpi::accounts::Initialize {
    user: self.user.to_account_info(),
    account: self.application_account.to_account_info(),  // field names come from the REGISTRY's IDL
    system_program: self.system_program.to_account_info(),
};
let registration_ctx = CpiContext::new(self.application_program.key(), registration_accounts);
registration::cpi::initialize(registration_ctx, GITHUB_HANDLE.to_string())?;
```

Anatomy: the accounts struct is *the callee's* form (names from its IDL); the `CpiContext` target is the registry's pubkey; the helper builds `[175,175,109,31,13,152,155,237] + borsh("rafael-ariascalles")` and invokes. No `signer_seeds`: the registry needs `user` as signer, and the user's signature on the outer transaction flows inward. The `?` enforces atomicity — if the registry rejects, the SOL transfer above it rolls back too.

`GITHUB_HANDLE` is a `#[constant]` in `constants.rs` — compiled into the binary (which itself lives in on-chain accounts). Correctness by construction: the handle can only change via redeploy or the registry's `update` instruction.

### Registry's instruction menu (from its IDL)

```
initialize(github)   creates ApplicationAccount                          ← our CPI
update(github)       changes the stored handle
submit_ts()          flips pre_req_ts, mints TS badge NFT (via Metaplex Core CPI)
submit_rs()          flips pre_req_rs, mints Rust badge NFT; requires pre_req_ts first
create_collection()  one-time course admin op
close()              deletes ApplicationAccount, refunds rent
errors: 6000 TsNotCompleted · 6001 TsAlreadyCompleted · 6002 RsAlreadyCompleted · 6003 RsNotInTimeWindow
```

## 8. Byte layouts

`ApplicationAccount` at the registry (as decoded from live devnet data):

```
0..8    discriminator    sha256("account:ApplicationAccount")[..8]
8..40   user             pubkey
40      bump             u8  (live value: 255)
41      pre_req_ts       bool
42      pre_req_rs       bool
43..47  github length    u32 LE
47..    github           utf-8  (live value: "rafael-ariascalles")
```

The GitHub string's full journey: **constant in the binary → serialized into the CPI instruction data (transient) → decoded by the registry's handler → written into the account (durable).** Transactions carry intent; accounts carry memory. Only the account survives to be read weeks later by `submit_rs`.

## 9. How the client consumes it (tests/pre-req-vault.ts)

The TS test demonstrates the full client pattern:

1. **Derive PDAs off-chain** with `findProgramAddressSync` — the same equation the program will check; client and program meet at the same address from opposite sides.
2. **Build the transaction**: `.methods.withdraw(new BN(amount)).accountsStrict({…}).rpc()` — discriminator + args serialized; account list must match the Rust struct field-for-field.
3. **Assert ledger state**: balances, and — after our addition — the contents of the registry account.

`Anchor.toml` sets `skip_local_validator = true` + `cluster = "devnet"`, so `anchor test` runs 4 mocha tests directly against devnet.

## 10. Course objectives — final checklist

> *Read through the Vault program and develop a high-level understanding: the purpose of each instruction; the accounts involved; how program state changes over time; the overall flow.*

- **Purpose of each instruction** — §5. `initialize` bootstraps per-user state; `deposit` funds the PDA bucket; `withdraw` releases funds with program authority and performs the cross-program registration; `close` tears down and refunds. Verified individually on devnet.
- **The accounts involved** — §4. Three PDAs across three owner programs (this program, system program, registry), plus the signer wallet and two program accounts. Each with a defined birth, owner, and death.
- **How state changes over time** — §6, backed by live transaction signatures. Accounts are born (`init` or first lamports), mutate (balances, bumps, flags), and die (drain-to-zero + `close`), all inside atomic transactions.
- **The overall flow** — client derives addresses → transaction passes validation (the `#[derive(Accounts)]` gauntlet) → handler logic → CPIs across ownership boundaries → ledger rows change. The vault exists so that *only the program* can release funds; the registration CPI exists because *only the registry* can write its own accounts. Composition is the architecture.

**Commentary:** what this exercise really teaches is that a Solana program is never an island. The vault is three ownership boundaries managed in one binary: its own data account, the system program's lamport domain, and a third-party registry. Every mechanism here — PDAs, bumps, `invoke_signed`, `declare_program!`, account validation — exists to make crossing those boundaries explicit and verifiable. The moment the withdraw transaction created `2D44dQv5…` while moving SOL back to the wallet, in one atomic step, the whole model stopped being theory.
