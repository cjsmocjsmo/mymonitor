
arch=$(uname -m)
if [ "$arch" = "armv7l" ]; then
    cp -pvr ./mymonitor-rpi3b /usr/local/bin/mymonitor-rpi3
    cp -pvr ./mymonitorrpi3.service /etc/systemd/system/mymonitorrpi3.service
    systemctl daemon-reload
    systemctl enable mymonitorrpi3.service
    systemctl start mymonitorrpi3.service
fi

if [ "$arch" = "aarch64" ] || [ "$arch" = "x86_64" ]; then
    cp -pvr ./mymonitor-rpi4 /usr/local/bin/mymonitor-rpi4
    cp -pvr ./mymonitorrpi4.service /etc/systemd/system/mymonitorrpi4.service
    systemctl daemon-reload
    systemctl enable mymonitorrpi4.service
    systemctl start mymonitorrpi4.service
fi