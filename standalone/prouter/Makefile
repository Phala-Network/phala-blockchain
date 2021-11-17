PI2P_Directory := ./i2pd
PI2P_Build_Directory := ./lib
PROUTER_BIN_DIRECTORY := ./bin

.PHONY: all
all:
	@rm -rf $(PI2P_Build_Directory)/*.a
	@cd $(PI2P_Directory) && USE_STATIC=yes make wrapper
	@cp $(PI2P_Directory)/*.a $(PI2P_Build_Directory)/
	@cargo build --release
	@cp ./target/release/prouter $(PROUTER_BIN_DIRECTORY)/

.PHONY: compile_i2pd
compile_i2pd:
	@rm -rf $(PI2P_Build_Directory) && mkdir $(PI2P_Build_Directory)
	@cd $(PI2P_Directory) && USE_STATIC=yes make wrapper
	@cp $(PI2P_Directory)/*.a $(PI2P_Build_Directory)/

.PHONY: clean
clean:
	@rm -rf $(PI2P_Build_Directory)
	@cd $(PI2P_Directory) && make clean
	@cargo clean
