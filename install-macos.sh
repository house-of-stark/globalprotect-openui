#!/usr/bin/env bash
# install-macos.sh — macOS (Homebrew) install helper for GP OpenUI.
#
# Called by install.sh when running on Darwin. Provides macOS-specific
# implementations for build deps, node install, and binary placement.
#
# On macOS, GP OpenUI uses Homebrew's openconnect binary directly for the
# VPN tunnel — no gpservice/gpclient daemon is needed.

set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BINARY_NAME="gpgui"

# ---------------------------------------------------------------------------
# Helpers (sourced caller may also define these; no-op if already defined)
# ---------------------------------------------------------------------------
if ! declare -f info &>/dev/null; then
  info()    { printf '\033[0;32m[INFO]\033[0m  %s\n' "$*"; }
  warn()    { printf '\033[0;33m[WARN]\033[0m  %s\n' "$*"; }
  error()   { printf '\033[0;31m[ERROR]\033[0m %s\n' "$*" >&2; }
  die()     { error "$*"; exit 1; }
fi
if ! declare -f have &>/dev/null; then
  have() { command -v "$1" &>/dev/null; }
fi

# ---------------------------------------------------------------------------
# 1. Build dependencies (macOS)
# ---------------------------------------------------------------------------
install_build_deps() {
  info "  On macOS the only system dependency is Xcode CLI tools."
  if ! xcode-select -p &>/dev/null; then
    info "  Installing Xcode Command Line Tools..."
    xcode-select --install
    info "  Please re-run the script after the installation completes."
    exit 0
  else
    info "  Xcode CLI tools already installed at: $(xcode-select -p)"
  fi

  info "  Checking Homebrew..."
  if ! have brew; then
    info "  Installing Homebrew..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
  fi

  info "  Checking openconnect..."
  if ! have openconnect; then
    info "  Installing openconnect via Homebrew..."
    brew install openconnect
  fi

  info "Build dependencies installed."
}

# ---------------------------------------------------------------------------
# 2. Node.js (macOS — via Homebrew) + pnpm
# ---------------------------------------------------------------------------
install_node() {
  if ! have node; then
    info "Node.js not found. Installing via Homebrew..."
    brew install node@22
    # brew node formulae are keg-only; link and set up PATH
    brew link --overwrite node@22
    export PATH="/opt/homebrew/opt/node@22/bin:$PATH"
  fi
  info "Node.js version: $(node --version)"
}

install_pnpm() {
  if have pnpm; then
    info "pnpm already installed ($(pnpm --version))."
    return
  fi
  info "Installing pnpm..."
  if have corepack; then
    corepack enable
    corepack prepare pnpm@latest --activate
  else
    npm install -g pnpm
  fi
  info "pnpm version: $(pnpm --version)"
}

# ---------------------------------------------------------------------------
# 3. GlobalProtect-openconnect (macOS — not needed)
# ---------------------------------------------------------------------------
install_gp_upstream() {
  info "GP OpenUI on macOS uses Homebrew's openconnect binary directly."
  info "No gpservice/gpclient daemon required — skipping upstream install."
}

# ---------------------------------------------------------------------------
# 4. Install the gpgui binary and open the .app bundle (macOS)
# ---------------------------------------------------------------------------
install_binary() {
  local src="$REPO_DIR/src-tauri/target/release/$BINARY_NAME"
  local dest="/usr/local/bin/$BINARY_NAME"

  info "Installing $BINARY_NAME to $dest..."
  sudo install -m 755 "$src" "$dest"
  info "Installed: $dest"

  # Also open the .app bundle if it exists
  local bundle_dir="$REPO_DIR/src-tauri/target/release/bundle/macos"
  local app_name="GP OpenUI.app"
  if [ -d "$bundle_dir/$app_name" ]; then
    info "Opening $app_name..."
    open "$bundle_dir/$app_name"
  fi
}
