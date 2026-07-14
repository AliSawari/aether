#!/usr/bin/env bash
# wgctl.sh — Aether WireGuard control (bundled)
# Usage:
#   ./wgctl.sh --workspace DIR u|up     [N]
#   ./wgctl.sh --workspace DIR d|down
#   ./wgctl.sh --workspace DIR r|restart [N]
#   ./wgctl.sh --workspace DIR status

set -euo pipefail

BASE_DIR=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --workspace)
      BASE_DIR="${2:?--workspace requires a path}"
      shift 2
      ;;
    *)
      break
      ;;
  esac
done

if [[ -z "${BASE_DIR}" ]]; then
  BASE_DIR="${AETHER_WORKSPACE:-}"
fi

if [[ -z "${BASE_DIR}" ]]; then
  echo "ERROR: workspace required via --workspace DIR or AETHER_WORKSPACE" >&2
  exit 1
fi

SUDO=""
[[ $EUID -ne 0 ]] && SUDO="sudo"
WGQ="wg-quick"

server_count() {
  local n=0
  for f in "$BASE_DIR"/smart*-wifi.conf; do
    [[ -f "$f" ]] && ((n++)) || true
  done
  echo "$n"
}

detect_iface_type() {
  local wg_iface
  wg_iface=$($SUDO wg show interfaces 2>/dev/null || true)

  if [[ -n "$wg_iface" ]]; then
    case "$wg_iface" in
      *wifi*) echo "wifi"; return ;;
      *eth*)  echo "eth"; return ;;
    esac
  fi

  local iface
  iface=$(ip route get 1.1.1.1 2>/dev/null \
    | awk '{for(i=1;i<=NF;i++) if($i=="dev"){print $(i+1); exit}}')

  case "$iface" in
    e*) echo "eth"  ;;
    w*) echo "wifi" ;;
    *)
      echo "ERROR: Unrecognized interface '${iface:-}' — not eth or wifi." >&2
      exit 1
      ;;
  esac
}

get_cfg_path() {
  local idx="$1"
  local type
  type=$(detect_iface_type)
  echo "$BASE_DIR/smart${idx}-${type}.conf"
}

get_active_cfg() {
  local iface
  iface=$($SUDO wg show interfaces 2>/dev/null || true)

  if [[ -z "$iface" ]]; then
    echo "ERROR: No active WireGuard interface found." >&2
    exit 1
  fi

  local cfg="$BASE_DIR/${iface}.conf"
  if [[ ! -f "$cfg" ]]; then
    echo "ERROR: Config file not found: $cfg" >&2
    exit 1
  fi
  echo "$cfg"
}

cmd_up() {
  local idx="$1"
  local cfg
  cfg=$(get_cfg_path "$idx")
  if [[ ! -f "$cfg" ]]; then
    echo "ERROR: Config not found: $cfg (run Regenerate first)" >&2
    exit 1
  fi
  echo "↑ Bringing up $cfg"
  $SUDO $WGQ up "$cfg"
}

cmd_down() {
  local cfg
  cfg=$(get_active_cfg)
  echo "↓ Bringing down $cfg"
  $SUDO $WGQ down "$cfg"
}

cmd_restart() {
  cmd_down || true
  cmd_up "$1"
}

cmd_status() {
  $SUDO wg show || true
}

validate_index() {
  local idx="$1"
  local max
  max=$(server_count)
  if ! [[ "$idx" =~ ^[0-9]+$ ]] || [[ "$idx" -lt 1 ]]; then
    echo "Invalid config index: $idx" >&2
    exit 2
  fi
  if [[ "$max" -eq 0 ]]; then
    echo "ERROR: No generated smart*-wifi.conf in $BASE_DIR (run Regenerate first)" >&2
    exit 1
  fi
  if [[ "$idx" -gt "$max" ]]; then
    echo "Invalid config index: $idx (choose 1-$max)" >&2
    exit 2
  fi
}

case "${1-}" in
  u|up)
    shift
    validate_index "${1:?Choose server index}"
    cmd_up "$1"
    ;;
  d|down)
    cmd_down
    ;;
  r|restart)
    shift
    validate_index "${1:?Choose server index}"
    cmd_restart "$1"
    ;;
  status|s)
    cmd_status
    ;;
  *)
    max=$(server_count)
    cat <<EOF
Usage:
  $(basename "$0") --workspace DIR u|up       [1-$max]
  $(basename "$0") --workspace DIR d|down
  $(basename "$0") --workspace DIR r|restart  [1-$max]
  $(basename "$0") --workspace DIR s|status
EOF
    ;;
esac
