#!/usr/bin/env bash
set -euo pipefail

#######################################################################
# bargo development bootstrap script
#
# – Installs rustup, nargo, bb, garaga, scarb, uv, starkli
# – Pins versions so CI/dev laptops behave the same
# – Writes everything to $HOME/.local/bin and ensures it’s on PATH
#######################################################################

### ===== versions you may bump later =====
NARGO_VERSION="1.0.0-beta.4"
BB_VERSION="0.87.4-starknet.1"
GARAGA_VERSION="0.18.1"
PYTHON_VERSION="3.10"
ASDF_VERSION="v0.18.0"     # used by starkup for scarb
UV_VERSION="0.7.13"

### ===== helpers =====
BIN_DIR="$HOME/.local/bin"
mkdir -p "$BIN_DIR"

need_path() {
  case ":$PATH:" in
    *":$BIN_DIR:"*) : ;;
    *)  echo "export PATH=\"$BIN_DIR:\$PATH\"" >> "$HOME/.$(basename "$SHELL")rc"
        export PATH="$BIN_DIR:$PATH"
        echo "🔹 Added $BIN_DIR to PATH (reload shell to persist)"
    ;;
  esac
}

fetch() {  # curl wrapper with progress & fail-fast
  curl -L --proto '=https' --tlsv1.2 -f "$1" -o "$2"
}

### ===== rust toolchain =====
if ! command -v cargo >/dev/null; then
  echo "🔧 Installing rustup..."
  curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh -s -- -y
  source "$HOME/.cargo/env"
fi
rustup update stable

### ===== asdf (needed by starkup for scarb) =====
if ! command -v asdf >/dev/null; then
  echo "🔧 Installing asdf $ASDF_VERSION…"
  git clone https://github.com/asdf-vm/asdf.git "$HOME/.asdf" --branch "$ASDF_VERSION"
  echo ". \"\$HOME/.asdf/asdf.sh\"" >> "$HOME/.$(basename "$SHELL")rc"
  . "$HOME/.asdf/asdf.sh"
fi

### ===== starkup → scarb, starknet-foundry, etc. =====
if ! command -v scarb >/dev/null; then
  echo "🔧 Installing scarb via starkup…"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.starkup.dev | bash -s -- -y
  need_path
fi

### ===== uv (python manager) =====
if ! command -v uv >/dev/null; then
  echo "🔧 Installing uv $UV_VERSION…"
  fetch "https://astral.sh/uv/install.sh" /tmp/uv_install.sh
  bash /tmp/uv_install.sh --version "$UV_VERSION"
  need_path
fi

### ===== Python $PYTHON_VERSION & virtual-env =====
if [ ! -d ".venv" ]; then
  echo "🔧 Creating Python $PYTHON_VERSION venv…"
  uv python install "$PYTHON_VERSION"
  uv venv
fi
source .venv/bin/activate

### ===== noirup / nargo =====
if ! command -v noirup >/dev/null; then
  echo "🔧 Installing noirup…"
  fetch https://raw.githubusercontent.com/noir-lang/noirup/main/install /tmp/noirup_install
  bash /tmp/noirup_install -y
  need_path
fi

echo "📦 Installing nargo $NARGO_VERSION"
noirup --version "$NARGO_VERSION"

### ===== bbup / bb =====
if ! command -v bbup >/dev/null; then
  echo "🔧 Installing bbup…"
  fetch https://raw.githubusercontent.com/AztecProtocol/aztec-packages/master/barretenberg/bbup/install /tmp/bbup_install
  bash /tmp/bbup_install -y
  need_path
fi

echo "📦 Installing bb $BB_VERSION"
bbup --version "$BB_VERSION"

### ===== garaga =====
pip install --upgrade "garaga==$GARAGA_VERSION"

### ===== starkli =====
if ! command -v starkli >/dev/null; then
  echo "🔧 Installing starkli…"
  curl -L https://get.starkli.sh | sh
  need_path
fi

### ===== build & install bargo =====
echo "🔨 Building bargo…"
cargo build --release
cargo install --path .

echo "✅ All tooling installed. Run 'bargo doctor' to verify."
