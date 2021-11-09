PI2P_Directory := ./i2pd
PI2P_Build_Directory := ./build

.PHONY: all
all:
	@rm -rf $(PI2P_Build_Directory) && mkdir $(PI2P_Build_Directory)
	@cd $(PI2P_Directory) && USE_STATIC=yes make wrapper
	@cp $(PI2P_Directory)/*.a $(PI2P_Build_Directory)/

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
