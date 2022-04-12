FORCE:
build-image: FORCE
	docker build -t python-app .

gen-rootfs: FORCE
	docker rm -f python-app || true;
	mkdir -p ../.base/python-app/rootfs && \
	docker create --name python-app python-app && \
	docker export python-app | tar -C ../.base/python-app/rootfs -xf -;
	docker rm -f python-app || true;
