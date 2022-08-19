FROM ubuntu:20.04

ARG CARGO_RELEASE

ENV TZ=Europe/Berlin
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

RUN apt-get update && apt-get install -y \
    tzdata

COPY services /services
ADD $CARGO_RELEASE/easycontainer_example_supervisor /easycontainer_example_supervisor

CMD ["/easycontainer_example_supervisor"]