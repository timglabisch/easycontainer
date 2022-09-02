dev_build_easycontainer:
	docker build -t easycontainer:dev .

dev_run_easycontainer: dev_build_easycontainer
	docker run -it --rm \
		-v /var/run/docker.sock:/var/run/docker.sock \
		-v "`pwd`:`pwd`" \
		--workdir "`pwd`" \
		--entrypoint=/usr/local/cargo/bin/easycontainer \
		easycontainer:dev \
		--container "`pwd`" \
		--docker-tag "xxxxx:dev" \
		easycontainer_example_supervisor

