set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

# Render the fixed example and keep its browser preview live while templates change.
dev:
    #!/usr/bin/env bash
    set -Eeuo pipefail
    set -m

    preview_pr="progressus-hk/agent-workspace#56"
    review_file="examples/review.json"
    output_dir="target/guided-review-preview"
    output_file="${output_dir}/index.html"

    for command in cargo cargo-watch pnpm; do
      if ! command -v "${command}" >/dev/null 2>&1; then
        echo "missing required command: ${command}" >&2
        exit 1
      fi
    done

    mkdir -p "${output_dir}"

    render=(
      cargo run --quiet -- generate "${preview_pr}"
      --review "${review_file}"
      --output "${output_file}"
    )
    "${render[@]}"

    cargo watch \
      --quiet \
      --postpone \
      --watch templates \
      --watch src \
      --watch "${review_file}" \
      -- "${render[@]}" &
    watcher_pid=$!

    pnpm dlx browser-sync@3.0.4 start \
      --server "${output_dir}" \
      --files "${output_file}" \
      --host 127.0.0.1 \
      --listen 127.0.0.1 \
      --port 3000 \
      --no-online \
      --no-ui \
      --no-notify &
    server_pid=$!

    cleanup() {
      trap - EXIT INT TERM
      kill -TERM -- "-${watcher_pid}" "-${server_pid}" >/dev/null 2>&1 || true
      wait "${watcher_pid}" "${server_pid}" >/dev/null 2>&1 || true
    }
    stop() {
      exit 0
    }
    trap cleanup EXIT
    trap stop INT TERM

    wait "${server_pid}"
