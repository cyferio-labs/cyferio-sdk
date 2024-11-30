#!/usr/bin/env bash
trap 'jobs -p | xargs -r kill' EXIT
echo 'Running: '\''cd crates/rollup/'\'''
cd crates/rollup/
if [ $? -ne 0 ]; then
    echo "Expected exit code 0, got $?"
    exit 1
fi

echo 'Running: '\''make clean-db'\'''
make clean-db
if [ $? -ne 0 ]; then
    echo "Expected exit code 0, got $?"
    exit 1
fi

echo 'Running: '\''unset SOV_PROVER_MODE'\'''
unset SOV_PROVER_MODE
if [ $? -ne 0 ]; then
    echo "Expected exit code 0, got $?"
    exit 1
fi

echo 'Running: '\''cargo run --bin node'\'''
output=$(mktemp)
cargo run --bin node &> $output &
background_process_pid=$!
echo "Waiting for process with PID: $background_process_pid"
until grep -q -i RPC $output
do       
  if ! ps $background_process_pid > /dev/null 
  then
    echo "The background process died died" >&2
    exit 1
  fi
  echo -n "."
  sleep 5
done

echo 'Running: '\''make test-create-token'\'''
make test-create-token
if [ $? -ne 0 ]; then
    echo "Expected exit code 0, got $?"
    exit 1
fi

echo 'Running: '\''curl -sS http://127.0.0.1:12346/ledger/txs/0xc4a09c4bc4e0a2425384de1f9d468070f4616f03d503367287774c7191ef25db | jq'\'''
curl -sS http://127.0.0.1:12346/ledger/txs/0xc4a09c4bc4e0a2425384de1f9d468070f4616f03d503367287774c7191ef25db | jq
if [ $? -ne 0 ]; then
    echo "Expected exit code 0, got $?"
    exit 1
fi

echo 'Running: '\''curl -sS http://127.0.0.1:12346/ledger/txs/0xc4a09c4bc4e0a2425384de1f9d468070f4616f03d503367287774c7191ef25db/events | jq'\'''
curl -sS http://127.0.0.1:12346/ledger/txs/0xc4a09c4bc4e0a2425384de1f9d468070f4616f03d503367287774c7191ef25db/events | jq
if [ $? -ne 0 ]; then
    echo "Expected exit code 0, got $?"
    exit 1
fi

echo 'Running: '\''curl -Ss http://127.0.0.1:12346/modules/bank/tokens/token_17zrpsyv06x7wmf2hg878gg5szwurckr3e2u77fvrdmanjhve8r2sj4jy42/total-supply | jq -c -M'\'''
output=$(curl -Ss http://127.0.0.1:12346/modules/bank/tokens/token_17zrpsyv06x7wmf2hg878gg5szwurckr3e2u77fvrdmanjhve8r2sj4jy42/total-supply | jq -c -M)
expected='{"data":{"amount":1000000,"token_id":"token_17zrpsyv06x7wmf2hg878gg5szwurckr3e2u77fvrdmanjhve8r2sj4jy42"},"meta":{}}
'
# Either of the two must be a substring of the other. This kinda protects us
# against whitespace differences, trimming, etc.
if ! [[ $output == *"$expected"* || $expected == *"$output"* ]]; then
    echo "'$expected' not found in text:"
    echo "'$output'"
    exit 1
fi

if [ $? -ne 0 ]; then
    echo "Expected exit code 0, got $?"
    exit 1
fi

echo 'Running: '\''curl -Ss http://127.0.0.1:12346/modules/bank/tokens/token_17zrpsyv06x7wmf2hg878gg5szwurckr3e2u77fvrdmanjhve8r2sj4jy42/total-supply | jq -c -M'\'''
output=$(curl -Ss http://127.0.0.1:12346/modules/bank/tokens/token_17zrpsyv06x7wmf2hg878gg5szwurckr3e2u77fvrdmanjhve8r2sj4jy42/total-supply | jq -c -M)
expected='{"data":{"amount":1000000,"token_id":"token_17zrpsyv06x7wmf2hg878gg5szwurckr3e2u77fvrdmanjhve8r2sj4jy42"},"meta":{}}
'
# Either of the two must be a substring of the other. This kinda protects us
# against whitespace differences, trimming, etc.
if ! [[ $output == *"$expected"* || $expected == *"$output"* ]]; then
    echo "'$expected' not found in text:"
    echo "'$output'"
    exit 1
fi

if [ $? -ne 0 ]; then
    echo "Expected exit code 0, got $?"
    exit 1
fi

echo "All tests passed!"; exit 0
