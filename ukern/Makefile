TARGET = i686-unknown-none
BUILD_MODE = debug
BUILD_DIR = ../target/${TARGET}/${BUILD_MODE}

.PHONY: debug-build release-build build iso all run clean doc

debug-build:
	cargo build -Zjson-target-spec

release-build:
	cargo build --release -Zjson-target-spec

build: ${BUILD_MODE}-build

iso: build
	grub-file --is-x86-multiboot ${BUILD_DIR}/ukern || (echo "Error: ukern executable is not Multiboot-capable" && false)
	mkdir -p ${BUILD_DIR}/isodir/boot/grub
	cp ${BUILD_DIR}/ukern ${BUILD_DIR}/isodir/boot
	cp src/grub.cfg ${BUILD_DIR}/isodir/boot/grub
	grub-mkrescue -o ${BUILD_DIR}/ukern.iso ${BUILD_DIR}/isodir

doc:
	cargo doc

all: iso doc
run: iso
	qemu-system-i386 -audiodev pa,id=pa -machine pcspk-audiodev=pa -drive id=ukern,format=raw,file=${BUILD_DIR}/ukern.iso,if=ide -drive id=disk,file=${BUILD_DIR}/disk.img,format=raw,if=ide
debug: iso
	qemu-system-i386 -s -S -audiodev pa,id=pa -machine pcspk-audiodev=pa -drive format=raw,file=${BUILD_DIR}/ukern.iso &
	gdb -x gdbinit

clean:
	cargo clean
