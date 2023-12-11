#!/bin/bash

SECONDS=0
TRACE_FILE=$1
NUM_ITERS=$2

pids=()

for i in `seq $NUM_ITERS`
do
  ~/pythia/workloads/create_delete_vm_2 "$TRACE_FILE$i" 1 &
  pids+=($!)
done

for pid in ${pids[@]}
do
    wait $pid
done

duration=$SECONDS
log "END: DURATION: $duration seconds"