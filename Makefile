TARGET = i686-unknown-none
BUILD_MODE = debug
BUILD_DIR = target/${TARGET}/${BUILD_MODE}

debug-build:
	cargo build

debug-iso: debug-build
	grub-file --is-x86-multiboot ${BUILD_DIR}/os || (echo "Error: os executable is not Multiboot-capable" && false)
	mkdir -p ${BUILD_DIR}/isodir/boot/grub
	cp ${BUILD_DIR}/os ${BUILD_DIR}/isodir/boot
	cp src/grub.cfg ${BUILD_DIR}/isodir/boot/grub
	grub-mkrescue -o ${BUILD_DIR}/os.iso ${BUILD_DIR}/isodir

debug: debug-iso

debug-run: debug
	qemu-system-i386 -drive format=raw,file=${BUILD_DIR}/os.iso

run: ${BUILD_MODE}-run
