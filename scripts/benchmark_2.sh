#!/bin/bash

source "./scripts/common.sh"

PROGRAM=$1

if [ -z "$1" ]; then
  PROGRAM=baseline
fi

echo "<--------------------------------------------------------------"
echo "Setting up RocksDB Application"
echo "-------------------------------------------------------------->//"
rocksdb_application_setup

echo "<--------------------------------------------------------------"
echo "Running $PROGRAM RocksDB application once to warmup"
echo "-------------------------------------------------------------->//"
set -x
rocksdb_application
set +x

if [ "$PROGRAM" != "baseline" ]; then
  echo "<------------------------- eBQL PROBE -------------------------"
  echo "Starting eBQL probe..."
  echo "-------------------------------------------------------------->"
  ebpf_probe

  wait "$APP_PID"
  echo "<--------------------------------------------------------------"
  echo "Done running RocksDB Application."
  echo ""
  echo "Cleaning up eBQL probe..."
  echo "-------------------------------------------------------------->"
  sudo kill -s SIGINT "$EBPF_PROBE_PID"
  wait "$EBPF_PROBE_PID"
  stty sane
else
  wait "$APP_PID"
fi

# TODO: too lazy to not output here, but should change later
clear_stats

for i in {1..25}; do
  echo "<--------------------------------------------------------------"
  echo "Running $PROGRAM RocksDB Application {$i}"
  echo "-------------------------------------------------------------->//"
  rocksdb_application

  if [ "$PROGRAM" != "baseline" ]; then
    echo "<------------------------- eBQL PROBE -------------------------"
    echo "Starting eBQL probe..."
    echo "-------------------------------------------------------------->"
    ebpf_probe

    wait "$APP_PID"
    echo "<--------------------------------------------------------------"
    echo "Done running RocksDB Application."
    echo ""
    echo "Cleaning up eBQL probe..."
    echo "-------------------------------------------------------------->"
    sudo kill -s SIGINT "$EBPF_PROBE_PID"
    wait "$EBPF_PROBE_PID"
    stty sane
  else
    wait "$APP_PID"
  fi

  # tail -1 < "$APP_STDOUT-$PROGRAM" >> $logFile
done
