add_executable(test_cached_namespace tests/test_cached_namespace.c core/lean_container.c)
add_executable(test_setup_lean_container tests/test_setup_lean_container.c core/lean_container.c)
add_executable(test_lean_container_template tests/test_lean_container_template.c core/lean_container.c)
add_executable(test_lean_container_pause tests/test_lean_container_pause.c core/lean_container.c)
add_executable(test_setup_lean_container_w_double_fork tests/test_setup_lean_container_w_double_fork.c core/lean_container.c)
