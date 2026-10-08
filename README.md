# GP OpenUI
An open-source graphical front-end for the [GlobalProtect-openconnect](https://github.com/yuezk/GlobalProtect-openconnect) CLI tools, supporting password and SSO (SAML) authentication.

> **⚠️ This is a fork** of [MagiShira/globalprotect-openconnect-openui](https://github.com/MagiShira/globalprotect-openconnect-openui). Upstream (yuezk/GlobalProtect-openconnect) dropped the `install.sh` release asset — this fork adapts the installer to use the current per-distro package repos (PPA / COPR / Arch extra). Pull requests are welcome.

> **Disclaimer:** This is an unofficial third-party client and is not affiliated with or endorsed by Palo Alto Networks. GlobalProtect is a registered trademark of Palo Alto Networks.

## Security Model

A common question: *Does GlobalProtect enforce security policies on my machine after I connect?*

**No.** GlobalProtect — like all SSL VPNs — is a **layer 3 tunnel** (IP packets through a virtual network interface). The server cannot:

- Execute code or scan files on your machine
- Verify what VPN client binary you're running
- Enforce post-connect host compliance

### About HIP (Host Information Profile)

HIP is real, and it's worth being precise about it, because there are two different modes:

1. **Agentless HIP (the common case)** — during the login handshake, the portal sends an XML profile asking the client to *report* its posture: OS, anti-virus, disk encryption, firewall state. The client answers, and the gateway enforces based on that **self-reported answer**. There is no host inspection — the server only sees whatever the client claims. This is the mode an open-source client participates in: it simply reports a compliant posture.

2. **Agent-based HIP (official GlobalProtect agent only)** — newer deployments push a background agent that collects *real* host data (file hashes, running AV) and submits a HIP report. Crucially, this agent runs **on your machine, on its own initiative**, and re-checks only when *it* decides to. The server can *request* a re-check but has no channel to force one — remove the agent, and there is nothing left for the server to enforce against.

**The key point:** HIP enforcement lives entirely in either (a) the self-reported handshake, or (b) a client-side agent you'd have to be running yourself. The server has **no mechanism to inspect or control your host mid-session**. 

The official client and this open-source implementation both complete the same GP protocol handshake (portal prelogin, SAML/SSO login, gateway authentication). **There is no binary attestation in the GP protocol** — the server trusts what the client reports.

Once the tunnel is established, the only controls the VPN gateway has are standard: route policies, DNS assignment, ACLs, and session timeouts. These apply equally regardless of which VPN software you use.

<p align="center">
  <img src="https://github.com/user-attachments/assets/f19f05b5-1818-4f48-98be-4fc5b83b605f" />
</p>

## About

GP OpenUI is a custom GUI for the `gpclient` / `gpservice` daemon stack from [GlobalProtect-openconnect](https://github.com/yuezk/GlobalProtect-openconnect). It replaces the upstream `gpgui` binary with a free and open-source interface built with [Tauri](https://tauri.app) (Rust + React).

### Why?

The upstream `gpgui` binary is proprietary-- you can't study, modify, or share it. For a VPN client handling your credentials and network traffic, that matters. GP OpenUI is [free software](https://www.fsf.org/about/what-is-free-software).

## Features

- **Password authentication** — standard username/password login
- **SSO/SAML authentication** — embedded WebView or external browser
- **Client certificate authentication** — PKCS#8 (`.pem`) and PKCS#12 (`.p12`/`.pfx`)
- **Cookie reuse** — stay logged in across SAML sessions
- **OS spoofing** — present as Linux, Windows, or macOS to the portal
- **HIP report submission** — with optional custom script path
- **Tunnel options** — disable IPv6, disable DTLS, custom MTU, VPNC script, reconnect timeout
- **Theme support** — light, dark, and system-follow modes
- **Settings window** — persistent per-user settings stored locally
- **Wayland and X11** — native Wayland support, X11 fallback

## Roadmap

- [ ] **Manual gateway selection** — the app currently auto-selects the first gateway returned by the portal
- [x] **System tray integration** — minimize to tray, tray icon menu
- [ ] **Auto-start on login** — launch with the system and connect automatically
- [ ] **Resume on wake** — reconnect automatically after the system wakes from sleep

## Installation

Run the install script as a regular user (it uses `sudo` internally where root is required):

```bash
./install.sh
```

The script will:

1. Install system build dependencies (varies by platform — Xcode CLI tools on macOS, WebKitGTK/GTK3 on Linux)
2. Install Rust (≥ 1.85) via `rustup` if not already present
3. Install Node.js LTS and pnpm if not already present
4. Install GlobalProtect-openconnect (`gpservice` + `gpclient`) from the upstream package repository
5. Build this Tauri app (`pnpm install` + `cargo tauri build`)
6. Replace the system `gpgui` binary with the newly built binary (the original is backed up as `gpgui.upstream`)

**Supported platforms:**
- **Linux:** Debian/Ubuntu, Fedora/RHEL, Arch Linux
- **macOS (experimental):** Homebrew — build deps are auto-installed, but the upstream `GlobalProtect-openconnect` package has no Homebrew formula and must be [built from source manually](https://github.com/yuezk/GlobalProtect-openconnect#installation).

## Usage

After installation, launch the GUI through your application launcher or via:

```bash
gpclient launch-gui
```

### Legacy TLS Configurations

Some GlobalProtect installations use older TLS configurations (e.g. deprecated ciphers or older protocol versions) that are rejected by the system OpenSSL by default. If the GUI fails to connect with a TLS handshake error, use the `--fix-openssl` flag:

```bash
gpclient --fix-openssl launch-gui
```


## Requirements

- **Linux:** X11 or Wayland, with `apt`, `dnf`, or `pacman`
- **macOS (experimental):** Homebrew, Xcode Command Line Tools
- Internet access to download build tools and the upstream GP packages

## License

GPL-2.0

