#!/bin/bash

if [ -z "$1" ]
then
    echo "Please specify the number of lambdas to run"
else
    for LAMBDAID in $(seq 1 $1)
    do
        npx serverless invoke -f rust-cli -d '{
            "addr": "10.0.1.211:25000",
            "duration": 10,
            "number": 1,
            "length": 4096,
            "rw_ratio": 100}' &
    done
fi