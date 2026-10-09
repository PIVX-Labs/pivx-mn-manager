# PIVX Masternode Manager

Manage PIVX masternodes without having to know anything about node hosting.

PIVX Masternode Manager is a cross-platform desktop app (with a browser-capable
web build) that walks you through setting up and running PIVX masternodes —
from provisioning a fresh VPS to casting governance votes.

## Features

- **One-click VPS provisioning** — connects over SSH and installs a dockerized
  `pivxd` node for you: dependencies, service setup, the lot. No terminal
  skills required.
- **Masternode status monitoring** — collateral and masternode state
  (`ENABLED`, `PRE_ENABLED`, `MISSING`, …) polled live from the PIVX network,
  per node.
- **Governance voting** — sign and submit budget votes locally from your
  masternode private key. Cryptography runs in-process; keys are never
  transmitted.
- **Multi-VPS portfolio** — group masternodes by server, add/remove nodes,
  and export your configuration as JSON.

## Browser vs desktop

| Capability | Web build | Desktop build |
| --- | --- | --- |
| Masternode status monitoring | ✅ | ✅ |
| Governance vote signing | ✅ | ✅ |
| Configuration export | Via browser download | Native save dialog |
| VPS provisioning over SSH | ❌ | ✅ |

The web build is useful for a quick look at monitoring and voting; full
provisioning requires the desktop app, where the Rust (Tauri) backend can
open SSH connections.

## Getting started

### Desktop app (recommended for masternode operators)

Prerequisites: [Node.js](https://nodejs.org) 18+,
[Rust](https://rustup.rs), and the
[Tauri v2 prerequisites](https://tauri.app/start/prerequisites/) for your OS.

```bash
npm install
npm run tauri build
```

The packaged app is written to `src-tauri/target/release/bundle/`.

### Development

```bash
npm install
npm run dev
```

For Tauri hot-reload development:

```bash
npm run tauri dev
```

## Security notes

- Masternode private keys are used **locally only** — vote signatures are
  produced on your device and never sent anywhere.
- VPS credentials are held in app memory for the duration of a session and
  are used solely to open SSH connections to your own servers.
- The app talks to a public PIVX RPC endpoint for status lookups; no
  account, wallet import, or balance-bearing key is required for that.

## Tech stack

- **Frontend:** Vue 3 + TypeScript + Vite + Bootstrap
- **Crypto/signing:** `@noble/secp256k1`, `@noble/hashes`
- **Desktop backend:** Tauri v2 (Rust) with `russh` for SSH provisioning

## License

See [LICENSE](LICENSE).
