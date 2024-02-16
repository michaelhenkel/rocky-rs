FROM ubuntu:23.04

RUN apt-get update && apt-get install -y \
     libibverbs1 \
     ibverbs-utils \
     librdmacm1 \
     libibumad3 \
     ibverbs-providers \
     rdma-core \
     libibverbs-dev \
     iproute2 \
     bash

COPY target/release/rocky-server /rocky-server
COPY target/release/rocky-client /rocky-client
