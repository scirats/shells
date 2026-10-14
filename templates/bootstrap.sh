#!/bin/bash
set -eux -o pipefail
mkdir -p /etc/cloud/cloud.cfg.d
echo 'preserve_hostname: true' > /etc/cloud/cloud.cfg.d/99-shell-preserve-hostname.cfg || true
hostnamectl set-hostname {{name}} || true
sed -i "s/lima-{{name}}/{{name}}/g" /etc/hosts || true
export DEBIAN_FRONTEND=noninteractive
if command -v apt-get >/dev/null 2>&1; then
  apt-get update -y
  apt-get install -y --no-install-recommends {{packages}} snapd
  systemctl enable --now snapd.socket
  snap wait system seed.loaded
  snap install helix --classic
elif command -v apk >/dev/null 2>&1; then
  apk add --no-cache {{packages}}
elif command -v dnf >/dev/null 2>&1; then
  dnf install -y {{packages}}
elif command -v zypper >/dev/null 2>&1; then
  zypper --non-interactive install {{packages}}
fi
