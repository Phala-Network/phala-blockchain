CONTRACT=target/ink/$(CONTRACT_NAME).contract
PREFIX ?= $(shell realpath ../dist)

.PHONY: all always-rerun clean install
all: $(CONTRACT)

$(CONTRACT): always-rerun
	cargo contract build

always-rerun:

clean:
	cargo clean

install: $(CONTRACT)
	cp $(CONTRACT) $(PREFIX)/$(CONTRACT_NAME).contract