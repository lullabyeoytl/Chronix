# Makefile for Chronix

include mk/config.mk
include mk/kernel.mk
include mk/fs.mk
include mk/qemu.mk
include mk/user.mk
include mk/tests.mk
include mk/utils.mk

build: $(KERNEL_BIN) #fs-img: should make fs-img first #env
	$(call building, "cp $(FS_IMG) to $(FS_IMG_COPY)")
	@cp $(FS_IMG) $(FS_IMG_COPY)

env:
	(rustup target list | grep "$(TARGET) (installed)") || rustup target add $(TARGET)
	cargo install cargo-binutils
	rustup component add rust-src
	rustup component add llvm-tools-preview

run-inner: qemu-version-check build
	$(QEMU) $(QEMU_DEV_ARGS) $(QEMU_RUN_ARGS)

run: run-inner

clean:
	@cd os && cargo clean
	@cd user && cargo clean
	@sudo rm -f $(FS_IMG)
	@sudo rm -f $(FS_IMG_COPY)
	@sudo rm -rf mnt
	@sudo rm -rf cross-compiler/kendryte-toolchain
	@make -C $(BUSY_BOX_DIR) clean
	@make -C $(LIBC_BENCH_DIR) clean
	@make -C $(IOZONE_DIR) clean
	@make -C $(UNIX_BENCH_DIR) clean

.PHONY: build env run-inner run clean $(KERNEL_BIN)