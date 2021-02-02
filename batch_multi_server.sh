#!/bin/bash

## declare an array of destinations
declare -a addrs=("10.0.1.254" "10.0.1.77")
json='{
    "addr": "ip:25000",
    "duration": 30,
    "number": 1,
    "length": 4096,
    "rw_ratio": 100
}'

if [ -z "$1" ]
then
    echo "Please specify the number of lambdas per server to invoke"
elif (( $1 % ${#addrs[@]} == 1 ))
then
    echo "Please provide a multiple"
else
    let count=$1-1
    for LAMBDAID in $(seq 0 $count)
    do
        let i=($LAMBDAID % ${#addrs[@]})
        # echo "running ${LAMBDAID} lambda invocation to ${addrs[$i]}:25000"
        json_i=`echo "$json" | sed "s/ip/${addrs[$i]}/"`
        # echo "sending $json_i"
        npx serverless invoke -f rust-cli -d "$json_i" > res/${LAMBDAID}.json &
    done
fi