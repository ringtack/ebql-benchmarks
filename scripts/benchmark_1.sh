#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
source "./scripts/common.sh"

trap "exit" INT TERM

echo "<--------------------------------------------------------------"
echo "Make sure you can sudo!"
echo "-------------------------------------------------------------->//"
sudo echo

echo "<--------------------------------------------------------------"
echo "Setting up RocksDB Application"
echo "-------------------------------------------------------------->//"
rocksdb_application_setup

echo "<--------------------------------------------------------------"
echo "Running RocksDB Application as baseline"
echo "-------------------------------------------------------------->//"
PROGRAM="baseline"
rocksdb_application

wait "$APP_PID"
echo "<--------------------------------------------------------------"
echo "Done running RocksDB Application."
echo "-------------------------------------------------------------->//"

echo "<--------------------------------------------------------------"
echo "Starting eBQL probe..."
echo "-------------------------------------------------------------->//"
ebql_probe

echo "<--------------------------------------------------------------"
echo "Running RocksDB Application with eBQL probe"
echo "-------------------------------------------------------------->//"
PROGRAM="probe"
rocksdb_application

wait "$APP_PID"
echo "<--------------------------------------------------------------"
echo "Done running RocksDB Application. Cleaning up eBQL probe..."
echo "-------------------------------------------------------------->//"
kill -INT "$EBQL_PROBE_PID"

echo "<--------------------------------------------------------------"
echo "Killing remaining jobs"
echo "-------------------------------------------------------------->//"
kill "$(jobs -p)"

echo "<--------------------------------------------------------------"
echo "Generating visualizations..."
echo "-------------------------------------------------------------->//"

python3 ./viz/plot_rocksdb_throughput.py \
  "$APP_THROUGHPUT"-baseline \
  "$APP_THROUGHPUT"-probe

python3 ./viz/plot_rocksdb_quantiles.py \
  "$APP_QUANTILES"-baseline \
  "$APP_QUANTILES"-probe
