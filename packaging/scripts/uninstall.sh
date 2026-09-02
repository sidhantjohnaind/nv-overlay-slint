#!/usr/bin/env bash
set -euo pipefail

BIN_NAME="nv-overlay-slint"
INSTALL_DIR="${HOME}/.local/bin"
SYSTEMD_DIR="${HOME}/.config/systemd/user"
AUTOSTART_DIR="${HOME}/.config/autostart"
APPS_DIR="${HOME}/.local/share/applications"

echo "Uninstalling NV-Overlay..."

systemctl --user stop nv-overlay.service 2>/dev/null || true
systemctl --user disable nv-overlay.service 2>/dev/null || true

rm -f "${INSTALL_DIR}/${BIN_NAME}"
rm -f "${SYSTEMD_DIR}/nv-overlay.service"
rm -f "${APPS_DIR}/nv-overlay.desktop"
rm -f "${AUTOSTART_DIR}/nv-overlay.desktop"

systemctl --user daemon-reload 2>/dev/null || true

echo "NV-Overlay uninstalled successfully."
