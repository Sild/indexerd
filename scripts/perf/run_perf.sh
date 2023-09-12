#!/bin/bash -e
user_dir="$(pwd)"
root_dir="$(git rev-parse --show-toplevel)"

cd "${root_dir}" || (echo "Fail to change dir" && exit 1)
cargo build --release
log_file="/tmp/indexerd_perf_$(date +%s).log"
RUST_LOG=warn cargo run --release > "${log_file}" 2>&1 &
indexerd_pid=$!
echo -n "waiting for server..."
while ! lsof -iTCP:8088 -sTCP:LISTEN -P -n | grep -q "indexerd"; do
  echo -n "."
done
echo ""
echo "server is ready, run wrk..."
# git clone https://github.com/wg/wrk && cd wrk && make && cp wrk /usr/local/bin
cd "$(dirname "$(realpath -- "$0")")" || (echo "Fail to change dir" && exit 1)
wrk_output=$(wrk -c 100 -d 5 -t 10 --latency --timeout=1s -s multiple-url-path.lua http://localhost:8088/ 2>&1)
kill -SIGINT ${indexerd_pid}
echo "${wrk_output}"
cd "${user_dir}" || (echo "fail to go back in your dir" && exit 0)
echo "indexerd logs can be found here: ${log_file}"