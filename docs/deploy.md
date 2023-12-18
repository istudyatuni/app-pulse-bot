Service unit, copy to `/etc/systemd/system/app-pulse-bot.service`

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

Run

```sh
systemctl enable app-pulse-bot
```

Helper to update app

```sh
#!/bin/bash
mv app-pulse-bot /usr/local/bin
systemctl restart app-pulse-bot.service
```
