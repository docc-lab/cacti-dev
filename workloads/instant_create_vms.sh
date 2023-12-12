#!/bin/bash

SECONDS=0
TRACE_FILE=$1
NUM_ITERS=$2

pids=()

for i in `seq $NUM_ITERS`
do
  openstack --os-profile Devstack1 server create "test_server$i" --flavor m1.tiny --image cirros --network flat-lan-1-net &
  pids+=($!)
done

for pid in ${pids[@]}
do
    wait $pid
done

duration=$SECONDS
log "END: DURATION: $duration seconds"