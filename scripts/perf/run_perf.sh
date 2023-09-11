#!/bin/bash

# git clone https://github.com/wg/wrk && cd wrk && make && cp wrk /usr/local/bin
cd "$(dirname "$(realpath -- "$0")")"
wrk -c 10 -d 2 -t 2 --latency --timeout=1s -s multiple-url-path.lua http://localhost:8089/
cd -
