#!/usr/bin/env bash
set -euo pipefail

# NV-Overlay Installer for Linux (AMD64, ARM64, RISC-V)

BIN_NAME="nv-overlay-slint"
INSTALL_DIR="${HOME}/.local/bin"
SYSTEMD_DIR="${HOME}/.config/systemd/user"
AUTOSTART_DIR="${HOME}/.config/autostart"
APPS_DIR="${HOME}/.local/share/applications"

echo "=========================================="
echo "  NV-Overlay (Slint Edition) Installer   "
echo "=========================================="

# Create target directories
mkdir -p "${INSTALL_DIR}" "${SYSTEMD_DIR}" "${AUTOSTART_DIR}" "${APPS_DIR}"

# Find binary
if [[ -f "./${BIN_NAME}" ]]; then
    SOURCE_BIN="./${BIN_NAME}"
elif [[ -f "./target/release/${BIN_NAME}" ]]; then
    SOURCE_BIN="./target/release/${BIN_NAME}"
else
    echo "Error: ${BIN_NAME} binary not found in current directory or target/release."
    exit 1
fi

# Install binary
echo "==> Installing ${BIN_NAME} to ${INSTALL_DIR}..."
cp -f "${SOURCE_BIN}" "${INSTALL_DIR}/${BIN_NAME}"
chmod +x "${INSTALL_DIR}/${BIN_NAME}"

# Install Systemd user service
if [[ -f "./packaging/systemd/nv-overlay.service" ]]; then
    echo "==> Installing systemd user service..."
    cp -f "./packaging/systemd/nv-overlay.service" "${SYSTEMD_DIR}/nv-overlay.service"
    systemctl --user daemon-reload || true
fi

# Install Desktop entry
if [[ -f "./packaging/desktop/nv-overlay.desktop" ]]; then
    echo "==> Installing desktop entry and autostart..."
    cp -f "./packaging/desktop/nv-overlay.desktop" "${APPS_DIR}/nv-overlay.desktop"
    cp -f "./packaging/desktop/nv-overlay.desktop" "${AUTOSTART_DIR}/nv-overlay.desktop"
fi

echo ""
echo "Installation completed successfully!"
echo ""
echo "To enable and start the background systemd service now, run:"
echo "  systemctl --user enable --now nv-overlay.service"
echo ""
echo "To run manually:"
echo "  ${INSTALL_DIR}/${BIN_NAME}"
echo ""
echo "Global shortcuts (bind in GNOME/KDE Settings):"
echo "  Toggle overlay: ${INSTALL_DIR}/${BIN_NAME} --toggle"
echo "  Cycle presets:  ${INSTALL_DIR}/${BIN_NAME} --cycle"
echo "  Quit overlay:   ${INSTALL_DIR}/${BIN_NAME} --quit"
