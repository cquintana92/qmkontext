#!/bin/bash

if ! command -v systemctl &> /dev/null; then
  echo "Could not find systemctl, not reloading"
  exit 0
fi

if [ "$EUID" -ne 0 ]; then
  systemctl daemon-reload
else
  if command -v sudo &> /dev/null; then
    echo "Could not find sudo, not reloading"
    exit 0
  else
    sudo systemctl daemon-reload
  fi
fi
