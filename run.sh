#!/bin/bash
# creates the new rust binary
cargo b --release
# set permission such that binary has permission
sudo setcap cap_net_admin=epi /home/hugoh/Desktop/rust/trust2/target/release/trust2
# run binary in background
/home/hugoh/Desktop/rust/trust2/target/release/trust2 &
#get running process of binary's pid
pid=$!

sudo ip addr add 192.168.0.1/24 dev tun0
sudo ip link set up dev tun0
trap "kill $pid" INT TERM
wait $pid