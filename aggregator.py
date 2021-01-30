# importing the module
import sys
import json 

total_iops = 0
p95_lats = []

if len(sys.argv) < 0:
    print("Please specify the number of results.")
    exit(1)

num = int(sys.argv[1])
for i in range(num):
    with open(f'res/{i}.json') as json_file: 
        data = json.load(json_file) 
        print("iops:", data['iops']) 
        total_iops += data['iops']
        print("p95:", data['p95'])
        p95_lats.append(data['p95'])

print("\noverall iops:", total_iops) 
print("avg p95 latency:", round(sum(p95_lats)/num))