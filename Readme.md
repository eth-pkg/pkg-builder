# Pkg-builder

## Prerequisities


If you are only debian install sbuild
```bash
sudo apt install sbuild
```

Install qemu virtual environment so you can build for bookworm. (sbuild is only available on debian)

```
bash scripts/qemu-setup.sh # Only first time needs to be installed
virsh start sbuild
virst console sbuild # username: debian, password: debian 
```