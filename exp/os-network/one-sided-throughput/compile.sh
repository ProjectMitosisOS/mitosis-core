#!/usr/bin/env bash
pushd ../../../os-network/bench \
&& python3 build.py dc_read_server \
&& python3 build.py dc_read_client \
&& popd \
&& cp ../../../os-network/bench/dc_read_server_tests.ko . \
&& cp ../../../os-network/bench/dc_read_client_tests.ko .
