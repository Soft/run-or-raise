DESTDIR =
PREFIX = /usr/local
CARGO_FLAGS =

.PHONY: all target/release/run-or-raise install help

all: target/release/run-or-raise

target/release/run-or-raise:
	cargo build --release $(CARGO_FLAGS)

install: target/release/run-or-raise
	install -s -D -m755 -- target/release/run-or-raise "$(DESTDIR)$(PREFIX)/bin/run-or-raise"
	install -D -m644 -- man/run-or-raise.1 "$(DESTDIR)$(PREFIX)/share/man/man1/run-or-raise.1"

help:
	@echo "Available make targets:"
	@echo "  all      - Build run-or-raise (default)"
	@echo "  install  - Build and install run-or-raise and manual pages"
	@echo "  help     - Print this help"
