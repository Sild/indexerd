#!/bin/bash
script_dir="$(dirname "$(realpath -- "$0")")"
${script_dir}/run_perf.sh http://indexerd:8088/
