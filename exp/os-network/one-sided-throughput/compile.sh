#!/usr/bin/env bash
pushd ../../../os-network/bench \
&& python3 build.py one_sided_read_server \
&& python3 build.py dc_read_client \
&& python3 build.py rc_read_client \
&& popd \
&& cp ../../../os-network/bench/one_sided_read_server_tests.ko . \
&& cp ../../../os-network/bench/dc_read_client_tests.ko . \
&& cp ../../../os-network/bench/rc_read_client_tests.ko .
