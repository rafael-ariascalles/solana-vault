# Pre-req Vault Map

Three diagrams of how the vault program fits together. Companion page to
[Understanding the Pre-req Vault](understanding-the-vault.md) — that document has the prose,
on-chain evidence and byte layouts; this one has the pictures.

Program: `Cdew9Y89YgSVS184JFdo38ouTQbaji8rkzUAUUjdKZGs` (devnet) · Anchor 1.2.1

## 1. One transaction, end to end — `deposit` as the example

```mermaid
flowchart LR
    A["TS client<br/>tests/*.ts<br/>methods.deposit()<br/>wallet signs"]
        -- "submit tx (.rpc)" --> B["Solana runtime<br/>entrypoint()<br/>loads program at declare_id!"]
    B -- "raw ix bytes" --> C["#[program]<br/>first 8 bytes = discriminator<br/>→ deposit fn"]
    C -- "Context" --> D["#[derive(Accounts)]<br/>signer? seeds?<br/>mut? init? close?"]
    D -- "validated" --> E["handler<br/>ctx.accounts.deposit(amount)<br/>deposit.rs:29"]
    E -- "CPI call" --> F["system program<br/>transfer CPI<br/>vault += amount"]
    D -. "any check fails → transaction rejected,<br/>handler never runs" .-> A
```

Everything from `runtime` onward runs inside the deployed program; nothing before `handler` is
hand-written code. Anchor generated the entrypoint, the discriminator routing and the account
checks from the macros in `lib.rs` and `deposit.rs`. The dashed path is why the account struct
doubles as the program's access-control layer: validation failure stops the transaction before
any lamports can move.

## 2. The accounts, and who can sign for each

```mermaid
flowchart LR
    U["user wallet<br/>real keypair — can sign<br/>pays rent + fees<br/>receives refunds"]
    VS["vault_state — PDA<br/>seeds: [&quot;state&quot;, user.key()]<br/>owner: this program<br/>data: 8B disc + vault_bump + state_bump"]
    V["vault — PDA<br/>seeds: [&quot;vault&quot;, vault_state.key()]<br/>owner: system program<br/>SOL balance only — no private key exists"]
    R["application program (registry)<br/>Q3PreReqsRs · TRBZyQHB…<br/>imported via declare_program!(registration)"]

    U -- "initialize: creates it, payer = user" --> VS
    VS -. "close: deletes vault_state,<br/>rent refunded" .-> U
    VS -- "every ix re-derives vault address<br/>from seeds + the stored bump" --> V
    U -- "deposit — the wallet signs" --> V
    V -- "withdraw / close — the program signs<br/>with PDA seeds" --> U
    VS -.->|"withdraw CPIs into this program (implemented)"| R

    linkStyle 3 stroke:#048a5f,stroke-width:2px
    linkStyle 4 stroke:#6841c8,stroke-width:2px
```

Green edge: lamports move with a wallet signature. Violet edge: lamports move with a PDA-seed
"signature" (`CpiContext::new_with_signer`, `withdraw.rs:33-36`, `close.rs:37-45`). The security
model in one picture: the vault address is derived from `["vault", vault_state.key()]`, so no
keypair anywhere can sign for it — but this program can produce a valid signature at runtime by
supplying those seeds plus the stored bump. Anyone can deposit; only the program's own logic can
release. The registry edge (dashed) is the cross-program call added in
`withdraw.rs:46-57` — verified live: account
[`2D44dQv5…`](https://explorer.solana.com/address/2D44dQv5d6RwuH3pb5fFVqsg6W8fVPEwTJLYzmmZ8vmj?cluster=devnet)
now exists on devnet with `github: "rafael-ariascalles"`.

## 3. Lifecycle of a user's vault

```mermaid
flowchart LR
    I["1 · initialize<br/>creates vault_state<br/>stores both bumps"]
        -- "first" --> D["2 · deposit<br/>vault += amount<br/>wallet-signed CPI"]
    D -- "any order, any count" --> W["3 · withdraw<br/>vault -= amount<br/>PDA-signed CPI<br/>+ registry CPI"]
    W -- "any order, any count" --> D
    W -- "when done — terminal" --> C["4 · close<br/>vault_state deleted<br/>rent + balance → user"]
```

This is exactly the sequence `tests/pre-req-vault.ts` executes against devnet
(`Anchor.toml` has `skip_local_validator = true`), asserting each ledger effect: bumps stored
after step 1, balances shifting after 2 and 3, both account rows gone after 4 — verified on-chain
2026-10-20; see the [transaction table](understanding-the-vault.md#0-the-on-chain-evidence).
