FROM docker.io/library/ubuntu:22.04

# show backtraces
ENV RUST_BACKTRACE 1

# install tools and dependencies
RUN apt-get update && \
	DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
		ca-certificates && \
# apt cleanup
	apt-get autoremove -y && \
	apt-get clean && \
	find /var/lib/apt/lists/ -type f -not -name lock -delete; \
# add user and link ~/.local/share/polkadot to /data
	useradd -m -u 1000 -U -s /bin/sh -d /logos-dev logos-dev && \
	mkdir -p /data /logos-dev/.local/share && \
	chown -R logos-dev:logos-dev /data && \
	ln -s /data /logos-dev/.local/share/dev-node

USER logos-dev

# copy the compiled binary to the container
COPY --chown=logos-dev:logos-dev --chmod=774 target/release/dev-node /usr/bin/dev-node

# check if executable works in this container
RUN /usr/bin/dev-node --version

# ws_port
EXPOSE 9930 9333 9944 30333 30334

CMD ["/usr/bin/dev-node"]
