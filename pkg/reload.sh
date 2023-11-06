#!/bin/bash

SERVICE_PATH="/etc/systemd/system/qmkontext.service"
SERVICE_INSTALL_PATH="/usr/share/qmkontext/qmkontext.service"

if ! command -v systemctl &> /dev/null; then
  echo "Could not find systemctl, not reloading"
  exit 0
fi

if [ "$EUID" -eq 0 ]; then
  if [ ! -f "${SERVICE_PATH}" ]; then
    cp "${SERVICE_INSTALL_PATH}" "${SERVICE_PATH}"
    systemctl daemon-reload
  else
    echo "Service already existed, not replacing"
  fi
else
  if command -v sudo &> /dev/null; then
    echo "Could not find sudo, not reloading"
    exit 0
  else
    if [ ! -f "${SERVICE_PATH}" ]; then
        sudo cp "${SERVICE_INSTALL_PATH}" "${SERVICE_PATH}"
        sudo systemctl daemon-reload
      else
        echo "Service already existed, not replacing"
      fi
  fi
fi
