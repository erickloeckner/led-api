#!/bin/bash

dir=$(pwd)
user=$(whoami)
echo -e "[Unit]\nDescription=LED API\n\n[Service]\nExecStart=$dir/target/release/led-api $dir/config.toml\nUser=$user\nWorkingDirectory=$dir\n\n[Install]\nWantedBy=multi-user.target" > ./led-api.service
