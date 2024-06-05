#!/bin/bash

source "./scripts/common.sh"

echo "<--------------------------------------------------------------"
echo "Make sure you can sudo!"
echo "-------------------------------------------------------------->"
sudo echo

logFile=ebql_probe.log
echo "<--------------------------------------------------------------"
echo "Creating log file at $logFile"
echo "-------------------------------------------------------------->//"
echo "" > "$logFile"

echo "<--------------------------------------------------------------"
echo "Setting up RocksDB Application"
echo "-------------------------------------------------------------->//"
rocksdb_application_setup

for i in {1..20}; do
  echo "<------------------------- eBQL PROBE -------------------------"
  echo "({$i}) Starting eBQL probe... "
  echo "-------------------------------------------------------------->"
  ebql_probe

  echo "<--------------------------------------------------------------"
  echo "Running RocksDB Application with probe"
  echo "-------------------------------------------------------------->//"
  PROGRAM="probe"
  rocksdb_application

  wait "$APP_PID"
  sudo kill -s SIGINT "$EBQL_PROBE_PID"
  wait "$EBQL_PROBE_PID"

  tail -2 < "$APP_STDOUT-$PROGRAM" >> $logFile
done

echo "<--------------------------------------------------------------"
echo "Output log file:"
cat $logFile
echo "-------------------------------------------------------------->//"