To build and deploy, run:

```sh
# assuming you're logging in as root user
# build-deploy args: [host] [dir to place binary] [command to run after copying binary]
just build-deploy root@[host] /root '/root/app-pulse-bot install'
```

this will:

- build bot as static binary
- move binary to an appropriate location
- run bot as a systemd service
