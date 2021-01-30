#!/bin/bash

rm sg_set.sh
i=0
while read p; do
  sg_name="SECURITY_GROUP_ID_$i"
  echo "export $sg_name=$p" 
  echo "export $sg_name=$p" >> sg_set.sh
  let i=$i+1
done < sg_examples.txt
# source sg_set.sh
# rm sg_set.sh
