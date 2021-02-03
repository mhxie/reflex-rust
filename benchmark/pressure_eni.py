from asyncio import tasks
# from boto3 import client as boto3_client
import aioboto3
import json
import asyncio
import sys
import time
import ast
import os

# lambda_client = boto3_client('lambda')
# sqs_client = boto3_client('sqs')

# ips = ["10.0.1.254", "10.0.1.77"]
ips = ["10.0.1.208"]
port = 25000
name = "reflex-dev-rust-cli"
args = {
    "addr": "10.0.1.208:25000",
    "duration": 10,
    "number": 1,
    "length": 4096,
    "rw_ratio": 100}


async def batch_invoke(num):
    results = []
    
    async with aioboto3.client('lambda') as lambda_cli:
        tasks = []
        for i in range(num):
            args["addr"] = ips[i%len(ips)]+':'+str(port)
            # print(args)
            body = json.dumps(args)
            task = lambda_cli.invoke(
                FunctionName=name,
                InvocationType='RequestResponse',
                Payload=body,
            )
            tasks.append(task)
        start = time.time()
        rets = await asyncio.gather(*tasks)
        elapsed = time.time() - start
        print(f"Total elapsed time is {elapsed:.2f} seconds.")
        readers = [ret['Payload'].read() for ret in rets]
        results = await asyncio.gather(*readers)
    iops = 0
    p95 = 0   

    for result in results:
        dict_str = result.decode("UTF-8")
        res = ast.literal_eval(dict_str)
        iops += res["iops"]
        p95 += res["p95"]
    print(f"Total iops is {iops}, average p95 is {p95/num:.2f}us")

async def batch_invoke_sqs(num):
    async with aioboto3.client('sqs') as sqs_cli:
        tasks = []
        for i in range(num):
            args["addr"] = ips[i%len(ips)]+':'+str(port)
            # print(args)
            body = json.dumps(args)
            print(body)
            task = sqs_cli.send_message(
                QueueUrl="https://sqs.us-west-2.amazonaws.com/029353178123/reflex-sqs",
                MessageBody=body
            )
            tasks.append(task)
        start = time.time()
        rets = await asyncio.gather(*tasks)
        elapsed = time.time() - start
        print(f"Total elapsed time is {elapsed:.2f} seconds.")

if len(sys.argv) > 1:
    # asyncio.run(batch_invoke(int(sys.argv[1])))
    asyncio.run(batch_invoke_sqs(int(sys.argv[1])))
else:
    print("Please provide the num")