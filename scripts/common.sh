#!/bin/bash

PROGRAM=""

OUT_DIR="/nvme/rtang/ebql-benchmarks"

APP_THROUGHPUT="$OUT_DIR/app-throughput"
APP_QUANTILES="$OUT_DIR/app-quantiles"
APP_STDOUT="$OUT_DIR/app-stdout"
APP_STATS="$OUT_DIR/app-stats"
APP_DATA="$OUT_DIR/tmp-data"
# `lscpu -p` to see cores. To find real cores:
# `cat /sys/devices/system/cpu/cpu0/topology/thread_siblings_list`
# The first number is the real core.
CORE_LIST="0,1,2,3,4,5,6,7"
# CORE_LIST="0,1,2,3,4,5,6,7,8,9,10,11"
APP_PID=0

EBPF_PROBE_STDOUT="$OUT_DIR/probe-stdout"
PROBE_STATS="$OUT_DIR/probe-stats"
EBPF_PROBE_PID=0

function rocksdb_application_setup {
  if [ ! -d "$APP_DATA" ]; then
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
  else
    echo "DB directory $APP_DATA already exists; no setup needed..."
  fi
}

function rocksdb_application {
  THROUGHPUT_FILE="$APP_THROUGHPUT-$PROGRAM"
  QUANTILES_FILE="$APP_QUANTILES-$PROGRAM"
  STDOUT_FILE="$APP_STDOUT-$PROGRAM"
  STATS_FILE="$APP_STATS-$PROGRAM"

  APP_CMD="/home/rtang/dev/ebql-benchmarks/target/release/rocksdb-application"
  APP_CMD="$APP_CMD --throughput-path $THROUGHPUT_FILE"
  APP_CMD="$APP_CMD --quantiles-path $QUANTILES_FILE"
  APP_CMD="$APP_CMD --db-path $APP_DATA"
  APP_CMD="$APP_CMD --read-percent 1."
  APP_CMD="$APP_CMD --writer-threads 8"
  APP_CMD="$APP_CMD --data-size 128"
  APP_CMD="$APP_CMD --key-space 1000000"
  APP_CMD="$APP_CMD --num-ops 50000000"
  APP_CMD="$APP_CMD --delay-secs 0"
  APP_CMD="$APP_CMD --stats-path $STATS_FILE"

  APP_CMD="taskset -c $CORE_LIST $APP_CMD"

  # shellcheck disable=SC2024
  # This doesn't matter, since we're redirecting $APP_CMD
  $APP_CMD &> "$STDOUT_FILE" &
  APP_PID=$!
  echo "RocksDB application PID: $APP_PID."
  echo "  Benchmark: $PROGRAM"
  echo "  Running on cores $CORE_LIST"
  echo "  Writing throughput to $THROUGHPUT_FILE"
  echo "  Writing quantiles to $QUANTILES_FILE"
  echo "  Writing stats to $STATS_FILE"
}

function ebpf_probe {
  STATS_FILE="$PROBE_STATS-$PROGRAM"

  PROBE_CMD="/home/rtang/dev/ebql-benchmarks/target/release/ebpf-probe-pread"
  PROBE_CMD="$PROBE_CMD --probe-type $PROGRAM"
  PROBE_CMD="$PROBE_CMD --stats-path $STATS_FILE"

  PROBE_CMD="taskset -c $CORE_LIST $PROBE_CMD"

  STDOUT_FILE="$EBPF_PROBE_STDOUT-$PROGRAM"
  # shellcheck disable=SC2024
  # This doesn't matter, since we're redirecting $PROBE_CMD
  sudo $PROBE_CMD &> "$STDOUT_FILE" &
  EBPF_PROBE_PID=$!
  # echo "$EBPF_PROBE_PID"
  # EBPF_PROBE_PID=$(ps -o pid= --ppid $EBPF_PROBE_PID)
  echo "eBQL Probe PID: $EBPF_PROBE_PID"
  echo "  Writing stats to $STATS_FILE and $STATS_FILE-bpf"
}

function clear_stats {
  sudo echo > "$APP_STATS-$PROGRAM"
  sudo echo > "$PROBE_STATS-$PROGRAM"
  sudo echo > "$PROBE_STATS-$PROGRAM-bpf"
}
