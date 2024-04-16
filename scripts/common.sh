#!/bin/bash

PROGRAM=""

OUT_DIR="/nvme/rtang/ebql-benchmarks"

APP_THROUGHPUT="$OUT_DIR/app-throughput"
APP_QUANTILES="$OUT_DIR/app-quantiles"
APP_STDOUT="$OUT_DIR/app-stdout"
APP_DATA="$OUT_DIR/tmp-data"
# `lscpu -p` to see cores. To find real cores:
# `cat /sys/devices/system/cpu/cpu0/topology/thread_siblings_list`
# The first number is the real core.
ROCKSDB_CORE_LIST="0,1,2,3,4,5,6,7"
APP_PID=0

EBQL_PROBE_STDOUT="$OUT_DIR/ebql-probe-stdout"
EBQL_PROBE_PID=0

function rocksdb_application_setup {
  APP_CMD="./target/release/rocksdb-application"
  APP_CMD="$APP_CMD --throughput-path dummy"
  APP_CMD="$APP_CMD --quantiles-path dummy"
  APP_CMD="$APP_CMD --db-path $APP_DATA"
  APP_CMD="$APP_CMD --writer-threads 8"
  APP_CMD="$APP_CMD --data-size 128"
  APP_CMD="$APP_CMD --key-space 1000000"
  APP_CMD="$APP_CMD --setup-db"

  sudo rm -rf "$APP_DATA"
  $APP_CMD
}

function rocksdb_application {
  THROUGHPUT_FILE="$APP_THROUGHPUT-$PROGRAM"
  QUANTILES_FILE="$APP_QUANTILES-$PROGRAM"
  STDOUT_FILE="$APP_STDOUT-$PROGRAM"

  APP_CMD="./target/release/rocksdb-application"
  APP_CMD="$APP_CMD --throughput-path $THROUGHPUT_FILE"
  APP_CMD="$APP_CMD --quantiles-path $QUANTILES_FILE"
  APP_CMD="$APP_CMD --db-path $APP_DATA"
  APP_CMD="$APP_CMD --read-percent 1."
  APP_CMD="$APP_CMD --writer-threads 8"
  APP_CMD="$APP_CMD --data-size 128"
  APP_CMD="$APP_CMD --key-space 1000000"
  APP_CMD="$APP_CMD --num-ops 20000000"
  APP_CMD="$APP_CMD --delay-secs 10"

  APP_CMD="taskset -c $ROCKSDB_CORE_LIST $APP_CMD"

  $APP_CMD &> "$STDOUT_FILE" &
  APP_PID=$!
  echo "RocksDB application PID: $APP_PID."
  echo "  Benchmark: $PROGRAM"
  echo "  Running on cores $ROCKSDB_CORE_LIST"
  echo "  Writing throughput to $THROUGHPUT_FILE"
  echo "  Writing quantiles to $QUANTILES_FILE"
}

function ebql_probe {
  PROBE_CMD="./target/release/ebql-probe-pread-1"

  # shellcheck disable=SC2024
  # This doesn't matter, since we're redirecting $PROBE_CMD
  sudo $PROBE_CMD &> "$EBQL_PROBE_STDOUT" &
  EBQL_PROBE_PID=$!
  echo "eBQL Probe PID: $EBQL_PROBE_PID"
}
