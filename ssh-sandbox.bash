#!/usr/bin/env bash

NAME="${SANDBOX_NAME:-ssh-sandbox}"
PORT="${SANDBOX_PORT:-2222}"
BASE="${SANDBOX_BASE:-ubuntu:24.04}"
USER_NAME="${SANDBOX_USER:-tester}"
IMAGE="${NAME}:latest"

KEY_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/.sandbox"
KEY="$KEY_DIR/id_ed25519"

SSH_OPTS=(-i "$KEY" -p "$PORT"
          -o StrictHostKeyChecking=no
          -o UserKnownHostsFile=/dev/null
          -o LogLevel=ERROR)

die() { echo "error: $*" >&2; exit 1; }

preflight() {
  command -v docker >/dev/null || die "docker not found"
  docker info >/dev/null 2>&1 || die "docker daemon not reachable"
  command -v ssh-keygen >/dev/null || die "ssh-keygen not found"
}

ensure_key() {
  if [[ ! -f "$KEY" ]]; then
    mkdir -p "$KEY_DIR"
    chmod 700 "$KEY_DIR"
    ssh-keygen -q -t ed25519 -N "" -C "$NAME" -f "$KEY"
    echo "==> Generated key: $KEY"
  fi
}

build_image() {
  if docker image inspect "$IMAGE" >/dev/null 2>&1; then
    return
  fi
  echo "==> Building image $IMAGE from $BASE (one-time)..."
  docker build -q -t "$IMAGE" - <<EOF >/dev/null
FROM $BASE
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update \\
 && apt-get install -y --no-install-recommends openssh-server sudo ca-certificates \\
 && rm -rf /var/lib/apt/lists/* \\
 && mkdir -p /run/sshd \\
 && (id ubuntu >/dev/null 2>&1 && userdel -r ubuntu || true) \\
 && useradd -m -s /bin/bash $USER_NAME \\
 && echo '$USER_NAME ALL=(ALL) NOPASSWD:ALL' > /etc/sudoers.d/$USER_NAME \\
 && chmod 440 /etc/sudoers.d/$USER_NAME \\
 && sed -i 's/^#\\?PasswordAuthentication.*/PasswordAuthentication no/' /etc/ssh/sshd_config
EXPOSE 22
CMD ["/usr/sbin/sshd", "-D", "-e"]
EOF
}

container_exists() { docker container inspect "$NAME" >/dev/null 2>&1; }
container_running() { [[ "$(docker container inspect -f '{{.State.Running}}' "$NAME" 2>/dev/null)" == "true" ]]; }

install_key() {
  docker exec -i "$NAME" bash -c "
    install -d -m 700 -o $USER_NAME -g $USER_NAME /home/$USER_NAME/.ssh &&
    cat > /home/$USER_NAME/.ssh/authorized_keys &&
    chown $USER_NAME:$USER_NAME /home/$USER_NAME/.ssh/authorized_keys &&
    chmod 600 /home/$USER_NAME/.ssh/authorized_keys
  " < "$KEY.pub"
}

wait_for_ssh() {
  echo -n "==> Waiting for sshd"
  for _ in $(seq 1 30); do
    if ssh-keyscan -p "$PORT" -T 1 127.0.0.1 >/dev/null 2>&1; then
      echo " ready"
      return
    fi
    echo -n "."
    sleep 1
  done
  echo
  docker logs "$NAME" 2>&1 | tail -n 20 >&2
  die "sshd did not come up"
}

print_info() {
  cat <<EOF

Sandbox ready: container '$NAME'

  ssh ${SSH_OPTS[*]} $USER_NAME@127.0.0.1

Or just run:   $0 ssh
Tear down:     $0 down
Fresh start:   $0 reset
EOF
}

cmd_up() {
  preflight
  ensure_key
  build_image
  if container_running; then
    echo "==> Container '$NAME' already running"
  else
    container_exists && docker rm -f "$NAME" >/dev/null
    echo "==> Starting container on 127.0.0.1:$PORT"
    docker run -d --name "$NAME" -p "127.0.0.1:$PORT:22" "$IMAGE" >/dev/null \
      || die "failed to start (is port $PORT already in use? try SANDBOX_PORT=2223)"
    install_key
  fi
  wait_for_ssh
  print_info
}

cmd_down() {
  preflight
  if container_exists; then
    docker rm -f "$NAME" >/dev/null
    echo "==> Removed '$NAME'"
  else
    echo "==> Nothing to remove"
  fi
}

cmd_ssh() {
  container_running || die "container not running; run '$0 up' first"
  exec ssh "${SSH_OPTS[@]}" "$USER_NAME@127.0.0.1"
}

cmd_status() {
  preflight
  docker ps -a --filter "name=^${NAME}$" --format 'table {{.Names}}\t{{.Status}}\t{{.Ports}}'
}

case "${1:-up}" in
  up)     cmd_up ;;
  down)   cmd_down ;;
  ssh)    cmd_ssh ;;
  status) cmd_status ;;
  reset)  cmd_down; cmd_up ;;
  -h|--help|help) sed -n '2,17p' "$0" ;;
  *)      die "unknown command '$1' (try: up, ssh, status, reset, down)" ;;
esac
