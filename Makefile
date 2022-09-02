dev_build_easycontainer:
	docker build -t easycontainer:dev .

dev_example_supervisor: dev_build_easycontainer
	docker run -it --rm \
		-v /var/run/docker.sock:/var/run/docker.sock \
		-v "`pwd`:`pwd`" \
		--workdir "`pwd`" \
		--entrypoint=/usr/local/cargo/bin/easycontainer \
		easycontainer:dev \
		--container "`pwd`" \
		--docker-tag "xxxxx:dev" \
		example_supervisor

dev_example_workspace: dev_build_easycontainer
	docker run -it --rm \
		-v /var/run/docker.sock:/var/run/docker.sock \
		-v "`pwd`:`pwd`" \
		--workdir "`pwd`/example_workspace" \
		--entrypoint=/usr/local/cargo/bin/easycontainer \
		easycontainer:dev \
		--container "`pwd`" \
		--docker-tag "yyyyy:dev" \
		.
	docker run --rm yyyyy:dev_arm64_v8

