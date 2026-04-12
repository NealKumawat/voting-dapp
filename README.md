# 💰 Donation Tracker — Soroban Smart Contract

> A transparent, on-chain donation and fund-usage registry built with [Soroban](https://soroban.stellar.org/) on the Stellar network.

---

## 📖 Project Description

**Donation Tracker** is a Soroban smart contract that brings radical transparency to charitable giving. Every donation is recorded permanently on the Stellar blockchain, and every time funds are used, the organization must log a detailed spending report — also on-chain and fully public. No more wondering where your money went.

The contract is designed to be deployed by an NGO, fundraising campaign, DAO treasury, or any organization that wants to prove to its donors that contributions are being managed responsibly. Because the Stellar ledger is public and immutable, anyone — donor, regulator, or curious bystander — can verify the full financial history at any time, without needing to trust a central authority.

---

## 🔍 What It Does

### For Donors
- Call `donate(donor, amount, message)` to record your contribution on-chain.
- Every donation is stored with your address, the amount (in stroops), an optional personal message, and a ledger timestamp.
- Your complete giving history is retrievable at any time via `get_donor_history(donor)`.

### For Organizations (Admin)
- After deployment, call `initialize(admin)` once to designate the managing address.
- Whenever funds are used, call `report_usage(description, amount)` to publish a permanent spending record — description of what was bought, how much was spent, and when.
- The contract enforces a hard rule: **reported spending can never exceed total donations raised.**

### For the Public
- Anyone can call `total_raised()`, `total_spent()`, and `remaining_funds()` to see the full financial picture instantly.
- Individual donation and usage records are readable via `get_donation(id)` and `get_usage(id)`.
- All key actions emit on-chain **events** (`donated`, `spent`, `init`, `newadmin`) that can be indexed by explorers or off-chain dashboards.

---

## ✨ Features

### 🔒 Immutable Audit Trail
Every donation and every spending report is stored in Soroban's persistent storage. Records cannot be modified or deleted after they are written — creating a tamper-proof financial history.

### 📊 Real-Time Transparency Metrics
Three global counters are always available on-chain:
- **Total Raised** — sum of all donations received
- **Total Spent** — sum of all admin-reported expenditures
- **Remaining Funds** — the unallocated balance (raised − spent)

### 👤 Donor History
Each donor's wallet address maps to an ordered list of their donation IDs, making it easy to build donor dashboards or calculate total contributions from any address.

### 🛡️ Authorization Enforcement
- Only the donor themselves can submit their own donation (via `require_auth`).
- Only the admin can call `report_usage` or `transfer_admin`.
- Soroban's native auth framework prevents spoofed or unauthorized calls.

### ⛔ Overspend Protection
The contract panics if a usage report would push reported spending above total donations received, preventing fraudulent or erroneous accounting entries.

### 📡 On-Chain Events
Key state transitions emit Soroban events:

| Event        | Trigger                          | Payload            |
|--------------|----------------------------------|--------------------|
| `init`       | Contract initialized             | admin address      |
| `donated`    | New donation recorded            | (id, amount)       |
| `spent`      | New fund-usage report logged     | (id, amount)       |
| `newadmin`   | Admin transferred to new address | new admin address  |

Events can be consumed by Stellar Horizon, custom indexers, or frontend apps for real-time notifications.

### 🔄 Admin Transfer
Admin rights can be safely passed to a new address via `transfer_admin(new_admin)`, supporting DAO handoffs or organizational changes without redeploying the contract.

### 🧪 Test Suite Included
The contract ships with a full Rust unit test suite covering:
- Initialization
- Single and multiple donations
- Donor history indexing
- Usage reporting and balance checks
- Rejection of zero-amount donations
- Rejection of overspending
- Admin transfer

---

## 🏗️ Project Structure

```
donation-tracker/
├── Cargo.toml          # Soroban SDK dependency + build profiles
└── src/
    └── lib.rs          # Contract logic + data types + tests
```

---

## 🚀 Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- Soroban CLI: `cargo install --locked soroban-cli`
- Add the WASM target: `rustup target add wasm32-unknown-unknown`

### Build

```bash
cargo build --target wasm32-unknown-unknown --release
```

The compiled WASM artifact will be at:
```
target/wasm32-unknown-unknown/release/donation_tracker.wasm
```

### Run Tests

```bash
cargo test --features testutils
```

### Deploy to Testnet

```bash
# Configure testnet identity
soroban keys generate --global alice --network testnet

# Deploy
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/donation_tracker.wasm \
  --source alice \
  --network testnet
```

### Invoke

```bash
# Initialize
soroban contract invoke --id <CONTRACT_ID> --source alice --network testnet \
  -- initialize --admin <ADMIN_ADDRESS>

# Make a donation (1 XLM = 10_000_000 stroops)
soroban contract invoke --id <CONTRACT_ID> --source donor --network testnet \
  -- donate --donor <DONOR_ADDRESS> --amount 10000000 --message '"First donation!"'

# Report fund usage
soroban contract invoke --id <CONTRACT_ID> --source alice --network testnet \
  -- report_usage --description '"Purchased water filters"' --amount 5000000

# Check balance
soroban contract invoke --id <CONTRACT_ID> --network testnet -- remaining_funds
```

---

## 📐 Data Model

### `Donation`
| Field       | Type      | Description                        |
|-------------|-----------|------------------------------------|
| `id`        | `u64`     | Auto-incrementing unique ID        |
| `donor`     | `Address` | Stellar address of the donor       |
| `amount`    | `i128`    | Amount in stroops                  |
| `message`   | `String`  | Optional donor note                |
| `timestamp` | `u64`     | Ledger timestamp at donation time  |

### `FundUsage`
| Field         | Type      | Description                        |
|---------------|-----------|------------------------------------|
| `id`          | `u64`     | Auto-incrementing unique ID        |
| `description` | `String`  | What the funds were used for       |
| `amount`      | `i128`    | Amount spent in stroops            |
| `reported_by` | `Address` | Admin address that filed the report|
| `timestamp`   | `u64`     | Ledger timestamp of the report     |

---

## 📜 License

MIT — free to use, fork, and build upon.

---

## 🤝 Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you'd like to change. Make sure to update tests accordingly.

---

> Built with ❤️ on Stellar · Powered by Soroban

Wallet Address- GDIWYUXGOMW4DXQZ2X4AKYAOQXYHRZTGAEG3AUCNCXTCOTFVYTUCFYKI

Contract Address- CAPQ4GT6BUZWQDNM3AE2AWABDZQQK4ME7WX3ZESDWRE7TIIYU6Q2RTCF

https://stellar.expert/explorer/testnet/contract/CAPQ4GT6BUZWQDNM3AE2AWABDZQQK4ME7WX3ZESDWRE7TIIYU6Q2RTCF

<img width="1919" height="948" alt="image" src="https://github.com/user-attachments/assets/64dd0e10-e449-4d28-a992-84b397456216" />
