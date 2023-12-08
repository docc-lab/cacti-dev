#!/bin/bash

pids=()

for i in `seq $1`
do
  ~/pythia/workloads/create_delete_ip.sh ~/junk_traces.txt 1 &
  pids+=($!)
done

for pid in ${pids[@]}
do
    wait $pid
done

