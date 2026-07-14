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

  # Prefer first smart* interface if multiple
  local name
  for name in $iface; do
    case "$name" in
      smart*)
        local cfg="$BASE_DIR/${name}.conf"
        if [[ -f "$cfg" ]]; then
          echo "$cfg"
          return 0
        fi
        ;;
    esac
  done

  local cfg="$BASE_DIR/${iface%% *}.conf"
  if [[ ! -f "$cfg" ]]; then
    echo "ERROR: Config file not found for active interface(s): $iface" >&2
    exit 1
  fi
  echo "$cfg"
}

# Make PostUp/PostDown idempotent on existing generated confs (no regenerate needed)
harden_conf() {
  local cfg="$1"
  $SUDO chmod 600 "$cfg" 2>/dev/null || chmod 600 "$cfg" 2>/dev/null || true

  if ! grep -q 'PostUp = ip route add ' "$cfg" 2>/dev/null; then
    return 0
  fi

  local tmp
  tmp=$(mktemp)
  sed \
    -e 's/PostUp = ip route add /PostUp = ip route replace /g' \
    -e 's/^\(PostDown = ip route del [^|]*\)$/\1 || true/' \
    "$cfg" > "$tmp"

  if [[ -w "$cfg" ]]; then
    cat "$tmp" > "$cfg"
  else
    $SUDO tee "$cfg" < "$tmp" >/dev/null
  fi
  rm -f "$tmp"
  $SUDO chmod 600 "$cfg" 2>/dev/null || chmod 600 "$cfg" 2>/dev/null || true
}

# Tear down any leftover smart* WG interfaces (partial/failed sessions)
force_down_all_smart() {
  local name cfg
  for name in $($SUDO wg show interfaces 2>/dev/null || true); do
    case "$name" in
      smart*)
        cfg="$BASE_DIR/${name}.conf"
        if [[ -f "$cfg" ]]; then
          $SUDO $WGQ down "$cfg" 2>/dev/null || true
        fi
        # If still present, yank the link
        if $SUDO ip link show "$name" &>/dev/null; then
          $SUDO ip link delete "$name" 2>/dev/null || true
        fi
        ;;
    esac
  done
}

cmd_up() {
  local idx="$1"
  local cfg
  cfg=$(get_cfg_path "$idx")
  if [[ ! -f "$cfg" ]]; then
    echo "ERROR: Config not found: $cfg (run Regenerate first)" >&2
    exit 1
  fi

  harden_conf "$cfg"

  local want
  want=$(basename "$cfg" .conf)

  # Already connected to this server
  if $SUDO wg show interfaces 2>/dev/null | grep -qw "$want"; then
    echo "Already connected: $want"
    return 0
  fi

  # Clear leftover tunnels so we never hit "already exists" / sticky routes
  force_down_all_smart

  echo "↑ Bringing up $cfg"
  local err
  set +e
  err=$($SUDO $WGQ up "$cfg" 2>&1)
  local rc=$?
  set -e

  if [[ $rc -eq 0 ]]; then
    echo "Connected: $want"
    return 0
  fi

  # Interface up despite noise (resolvconf etc.)
  if $SUDO wg show interfaces 2>/dev/null | grep -qw "$want"; then
    echo "Connected with warnings: $want"
    return 0
  fi

  # Retry once after force-clean (handles half-applied PostUp)
  force_down_all_smart
  set +e
  err=$($SUDO $WGQ up "$cfg" 2>&1)
  rc=$?
  set -e

  if [[ $rc -eq 0 ]] || $SUDO wg show interfaces 2>/dev/null | grep -qw "$want"; then
    echo "Connected: $want"
    return 0
  fi

  echo "$err" | grep -vE '^(\[#\]|Warning:)' | tail -n 12 >&2 || true
  echo "ERROR: wg-quick failed to bring up $want" >&2
  exit 1
}

cmd_down() {
  local cfg=""
  set +e
  cfg=$(get_active_cfg 2>/dev/null)
  set -e
  if [[ -n "$cfg" && -f "$cfg" ]]; then
    echo "↓ Bringing down $cfg"
    $SUDO $WGQ down "$cfg" 2>/dev/null || true
  fi
  force_down_all_smart
  echo "Disconnected"
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
