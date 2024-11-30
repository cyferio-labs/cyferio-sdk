#!/bin/sh
echo "Configuring toxiproxy for sequencer-0:\n"
#TOXIPROXY_HOST="localhost"
TOXIPROXY_HOST="toxiproxy"

curl -s -XPOST -d '{"name" : "sequencer-0", "listen" : "0.0.0.0:26659", "upstream" : "sequencer-0:26658"}' http://$TOXIPROXY_HOST:8474/proxies
#curl -s -XPOST -d '{"type": "latency", "toxicity": 0.95, "attributes": {"latency" : 4000, }}' http://$TOXIPROXY_HOST:8474/proxies/sequencer-0/toxics
curl -s -XPOST -d '{"type": "timeout", "toxicity": 0.8, "attributes": {"timeout": 30000}}' http://$TOXIPROXY_HOST:8474/proxies/sequencer-0/toxics

# Other options to consider
# latency
# slow_close
# reset_peer

echo "\n\n=====\nConfiguration is completed!"