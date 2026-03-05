#!/bin/sh
ch="$1"
i=1
while [ $i -le ${#ch} ]; do
  d=$(echo "$ch" | cut -c$i)
  luna-send -n 1 -f luna://com.webos.service.networkinput/sendSpecialKey '{"key":"'$d'","rcu":true}'
  sleep 0.5
  i=$((i+1))
done
sleep 0.5
luna-send -n 1 -f luna://com.webos.service.networkinput/sendSpecialKey '{"key":"ENTER","rcu":true}'
