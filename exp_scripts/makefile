### configurations ###

USER=
PWD=
PROJECT_PATH=projects/mos
PARENT_GID=fe80:0000:0000:0000:248a:0703:009c:7ca0
PARENT_HOST=val06
CHILD_HOSTS=val07
STR_CHILD_HOSTS='val07'

#USE_PROXY_COMMAND=false # true or false
USE_PROXY_COMMAND=true # true or false

## Plot related
PLOT=1 # If plot the target figure

### end of configurations ###


STR_ALL_HOSTS='${PARENT_HOST}',${STR_CHILD_HOSTS}
RUNNER_FLAG=-k=$(USE_PROXY_COMMAND)

HOSTS={'builder':[$(STR_ALL_HOSTS)] , \
'parent':['$(PARENT_HOST)'],\
'child':[${STR_CHILD_HOSTS}],\
}

PLACEHOLDER={'parent_gid': '$(PARENT_GID)'}


template_args="{\
'pwd':'$(PWD)','user':'$(USER)',\
'hosts':$(HOSTS),'path':'$(PROJECT_PATH)',\
'placeholder': $(PLACEHOLDER) } \
"

# Template path list
template-build-exp=templates-build/template-build-exp.toml
template-clean=templates-build/template-clean.toml
template-build-mitosis=templates-build/template-build-mitosis.toml
template-build-lean-container=templates-build/template-build-lean-container.toml


# Evaluation runner templates
template-run-micro-startup=templates-run/template-run-micro-startup.toml
template-run-micro-execution-prepare=templates-run/template-run-micro-execution-prepare.toml


run_out=out
build_out=run-build.toml

gen-clean:
	python toml_generator.py -f $(template-clean) -o . -d ${template_args}


define builder
	mkdir -p ${run_out}
	python toml_generator.py -f $(1) -o . -d ${template_args}
	python bootstrap.py -f $(build_out) ${RUNNER_FLAG}
endef

define runner
	rm -rf $(2)
	python toml_generator.py -f $(1) -o $(2) -d ${template_args}
endef

micro-function-startup:
	$(call runner, $(template-run-micro-startup), ${run_out}/$@)

micro-function-execution-prepare:
	$(call runner, $(template-run-micro-execution-prepare), ${run_out}/$@)



# Built all cpp
build-cpp:
	$(call builder, $(template-build-exp))

build-mitosis: build-cpp
	$(call builder, $(template-build-mitosis))

mount:
	$(call builder, $(template-build-lean-container))

clean: gen-clean
	pkill python; sleep 0.1
	python bootstrap.py -f $(build_out) ${RUNNER_FLAG}