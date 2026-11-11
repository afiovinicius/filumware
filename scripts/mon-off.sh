#!/bin/sh

sudo ip link set wlp1s0 down && sudo iw wlp1s0 set type managed && sudo ip link set wlp1s0 up && sudo systemctl start NetworkManager && iwconfig && echo "Interface active MANAGED."
