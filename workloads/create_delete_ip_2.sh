#HMAC_KEY=$1
#NETWORK_ID=$2
#TRACE_FILE=$3
#tmpfile=$4
#
#create_ip () {
#    # Returns the server id
#    local server_id
#    local trace_id
#    openstack --os-profile $HMAC_KEY floating ip create $NETWORK_ID &> $tmpfile
#    server_id=$(grep '| id' $tmpfile | awk '{print $6}')
#    trace_id=$(grep 'Trace ID:' $tmpfile | awk '{print $5}')
#    echo $trace_id >> $TRACE_FILE
#    echo $server_id
#}
#
#delete_ip () {
#    # Requires server id
#    local server_id
#    local trace_id
#    server_id=$1
#    openstack --os-profile $HMAC_KEY floating ip delete $server_id &> $tmpfile
#    trace_id=$(grep 'Trace ID:' $tmpfile | awk '{print $5}')
#    echo $trace_id >> $TRACE_FILE
#}
#
#list_ip () {
#    local trace_id
#    openstack --os-profile $HMAC_KEY floating ip list &> $tmpfile
#    trace_id=$(grep 'Trace ID:' $tmpfile | awk '{print $5}')
#    echo $trace_id >> $TRACE_FILE
#}
#
##iteration () {
##  log "Creating IP $i ..."
##  ip=$(create_ip)
##  log "Created "${ip}
##  sleep 2
##  list_ip
##  log "Listed IPs"
##  log "Deleting "${ip}...""
##  delete_ip $ip
##  log "Deleted "${ip}
##  sleep 2
##}
#
#log "Creating IP $5 ..."
#ip=$(create_ip)
#log "Created "${ip}
#sleep 2
#list_ip
#log "Listed IPs"
#log "Deleting "${ip}...""
#delete_ip $ip
#log "Deleted "${ip}
#sleep 2