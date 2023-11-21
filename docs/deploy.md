Service unit, copy to `/etc/systemd/system/`

```
[Unit]
Description=Apps pulse, telegram bot

[Service]
ExecStart=/usr/local/bin/app-pulse-bot
Restart=on-failure
RestartSec=1

[Install]
WantedBy=multi-user.target
```

Helper to update app

```sh
#!/bin/bash
systemctl stop app-pulse-bot.service
mv app-pulse-bot /usr/local/bin
systemctl start app-pulse-bot.service
```
