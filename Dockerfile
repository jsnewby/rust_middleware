FROM ubuntu:18.04

RUN apt-get update && apt-get -y install libpq5 libcurl3 libcurl3-gnutls

COPY ./target/debug/aepp_middleware /app/aepp-middleware
COPY Rocket.toml /app/Rocket.toml
ENTRYPOINT ["/app/aepp-middleware"]
CMD ["-p", "-s"]