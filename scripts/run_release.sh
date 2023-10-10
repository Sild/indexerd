#!/bin/bash
RUST_LOG="info" cargo run --release -- configs/test.json || true


