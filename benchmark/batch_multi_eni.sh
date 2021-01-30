#!/bin/bash

if [ -z "$1" ]
then
    echo "Please specify the number of invocation per lambda to run"
elif [ -z "$2" ]
then
    echo "Please specify the number of lambdas to run"
else
    for INVOKEID in $(seq 1 $1)
    do
        for CLIID in $(seq 1 $2)
        do
            LAMBDAID=$((($INVOKEID-1)*$2+$CLIID-1))
            echo "running ${LAMBDAID} lambda invocation"
            npx serverless invoke -f rust-cli-$CLIID -d '{
                "addr": "10.0.1.27:25000",
                "duration": 30,
                "number": 1,
                "length": 4096,
                "rw_ratio": 100}' > res/${LAMBDAID}.json &
        done
    done
fi