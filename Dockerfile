# Note: We don't use Alpine and its packaged Rust/Cargo because they're too often out of date,
# preventing them from being used to build Substrate/Polkadot.

FROM phusion/baseimage:0.10.2 as builder

ENV DEBIAN_FRONTEND=noninteractive
ENV RUST_TOOLCHAIN=nightly-2020-05-31

ARG PROFILE=release
WORKDIR /wormhole

COPY . /wormhole

RUN apt-get update && \
	apt-get dist-upgrade -y -o Dpkg::Options::="--force-confold" && \
	apt-get install -y cmake pkg-config libssl-dev git clang

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y && \
	export PATH="$PATH:$HOME/.cargo/bin" && \
	rustup toolchain install $RUST_TOOLCHAIN && \
	rustup target add wasm32-unknown-unknown --toolchain $RUST_TOOLCHAIN && \
	rustup default stable && \
	rustup default $RUST_TOOLCHAIN

RUN export PATH="$PATH:$HOME/.cargo/bin" && cargo build "--$PROFILE" -j3

# ===== SECOND STAGE ======

FROM phusion/baseimage:0.10.2
ARG PROFILE=release

RUN mv /usr/share/ca* /tmp && \
	rm -rf /usr/share/*  && \
	mv /tmp/ca-certificates /usr/share/ && \
	mkdir -p /root/.local/share/wormhole && \
	ln -s /root/.local/share/wormhole /data

COPY --from=builder /wormhole/target/$PROFILE/wormhole /usr/local/bin

# checks
RUN ldd /usr/local/bin/wormhole && \
	/usr/local/bin/wormhole --version

# Shrinking
RUN rm -rf /usr/lib/python* && \
	rm -rf /usr/bin /usr/sbin /usr/share/man

# USER wormhole # see above
EXPOSE 30333 9933 9944
VOLUME ["/data"]

CMD ["/usr/local/bin/wormhole"]
