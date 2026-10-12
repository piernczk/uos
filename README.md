# uos

An incomplete kernel for i686.

So far, it supports paging, PS/2 keyboards, VGA (text mode only) and (Parallel)
ATA. There is absolutely no userspace or actual functionality, just the
interfaces.

It was more of a learning project than an actual kernel. Expect lots of useless
boilerplate, weird docs and general lack of idea for the project.

# Building and running

You will need the following dependencies (Arch Linux packages):
- `grub`
- `lld`
- `mtools`
- `nasm`
- `qemu`
- `xorriso`

Build and run with:
```bash
cd ukern
make run
```

# License

The project is licensed under... yeah i forgot to add a license. kinda bonkers
