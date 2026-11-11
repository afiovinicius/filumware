#!/bin/sh

sudo systemctl stop NetworkManager && sudo ip link set wlp1s0 down && sudo iw wlp1s0 set type monitor && sudo ip link set wlp1s0 up && iwconfig && echo "Interface active MONITOR."
