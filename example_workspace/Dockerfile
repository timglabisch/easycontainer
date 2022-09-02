FROM ubuntu:20.04

ARG CARGO_RELEASE

ENV TZ=Europe/Berlin
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

RUN apt-get update && apt-get install -y \
    tzdata

ADD $CARGO_RELEASE/binary_a /binary_a
ADD $CARGO_RELEASE/binary_b /binary_b

CMD ["/bin/bash", "-c", "/binary_a && /binary_b"]