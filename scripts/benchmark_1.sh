#!/bin/bash

# SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
source "./scripts/common.sh"

echo "<--------------------------------------------------------------"
echo "Make sure you can sudo!"
echo "-------------------------------------------------------------->"
sudo echo

echo "<--------------------------------------------------------------"
echo "Setting up RocksDB Application"
echo "-------------------------------------------------------------->"
rocksdb_application_setup

echo "<-------------------------- BASELINE --------------------------"
echo "Running RocksDB Application as baseline"
echo "-------------------------------------------------------------->"
PROGRAM="baseline"
rocksdb_application
wait "$APP_PID"

echo "<--------------------------------------------------------------"
echo "Done running RocksDB Application."
echo "-------------------------------------------------------------->"

echo ""

echo "<------------------------- eBQL PROBE -------------------------"
echo "Starting eBQL probe..."
echo "-------------------------------------------------------------->"
PROGRAM="ebql"
ebpf_probe

echo "<--------------------------------------------------------------"
echo "Running RocksDB Application with eBQL probe"
echo "-------------------------------------------------------------->"
rocksdb_application

wait "$APP_PID"
echo "<--------------------------------------------------------------"
echo "Done running RocksDB Application."
echo ""
echo "Cleaning up eBQL probe..."
echo "-------------------------------------------------------------->"
sudo kill -s SIGINT "$EBPF_PROBE_PID"
wait "$EBPF_PROBE_PID"
stty sane

echo ""

echo "<------------------------- OPT PROBE --------------------------"
echo "Starting optimized probe..."
echo "-------------------------------------------------------------->"
PROGRAM="opt"
ebpf_probe

echo "<--------------------------------------------------------------"
echo "Running RocksDB Application with optimized probe"
echo "-------------------------------------------------------------->"
rocksdb_application

wait "$APP_PID"
echo "<--------------------------------------------------------------"
echo "Done running RocksDB Application."
echo ""
echo "Cleaning up optimized probe..."
echo "-------------------------------------------------------------->"
sudo kill -s SIGINT "$EBPF_PROBE_PID"
wait "$EBPF_PROBE_PID"
stty sane

echo "<------------------------ UNOPT PROBE -------------------------"
echo "Starting unoptimized probe..."
echo "-------------------------------------------------------------->"
PROGRAM="unopt"
ebpf_probe

echo "<--------------------------------------------------------------"
echo "Running RocksDB Application with unoptimized probe"
echo "-------------------------------------------------------------->"
rocksdb_application

wait "$APP_PID"
echo "<--------------------------------------------------------------"
echo "Done running RocksDB Application."
echo ""
echo "Cleaning up optimized probe..."
echo "-------------------------------------------------------------->"
sudo kill -s SIGINT "$EBPF_PROBE_PID"
wait "$EBPF_PROBE_PID"
stty sane

echo ""
echo "<-------------------- VISUALIZATIONS --------------------------"
echo "Generating visualizations..."
echo "-------------------------------------------------------------->"

python3 ./viz/plot_rocksdb_throughput.py \
  "$APP_THROUGHPUT"-baseline \
  "$APP_THROUGHPUT"-ebql \
  "$APP_THROUGHPUT"-unopt \
  "$APP_THROUGHPUT"-opt

python3 ./viz/plot_rocksdb_quantiles.py \
  "$APP_QUANTILES"-baseline \
  "$APP_QUANTILES"-ebql \
  "$APP_QUANTILES"-unopt \
  "$APP_QUANTILES"-opt


echo "<---------------------- CLEANUP -------------------------------"
echo "Killing spawned jobs"
echo "-------------------------------------------------------------->"
sudo kill "$(jobs -p)"
stty sane

echo "<--------------------------------------------------------------"
echo "Background jobs after cleanup:"
echo "-------------------------------------------------------------->"
jobs

echo "<--------------------------------------------------------------"
echo "Output of ps -a after cleanup:"
echo "-------------------------------------------------------------->"
ps -a

echo "<--------------------------------------------------------------"
echo "Done cleaning, exiting"
echo "-------------------------------------------------------------->"