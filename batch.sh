#!/bin/bash

if [ -z "$1" ]
then
    echo "Please specify the number of lambdas to invoke"
else
    let count=$1-1
    for LAMBDAID in $(seq 0 $count)
    do
        echo "running ${LAMBDAID} lambda invocation"
        serverless invoke -f rust-cli -d '{
            "addr": "10.0.1.208:25000",
            "start": 0,
            "duration": 20,
            "number": 1,
            "length": 4096,
            "rw_ratio": 100}' > res/${LAMBDAID}.json &
    done
fi