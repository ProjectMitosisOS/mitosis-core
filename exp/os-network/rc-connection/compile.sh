pushd ../../../os-network/bench \
&& python3 build.py rc_conn_server \
&& python3 build.py rc_conn_client \
&& popd \
&& cp ../../../os-network/bench/rc_conn_server_tests.ko . \
&& cp ../../../os-network/bench/rc_conn_client_tests.ko .
