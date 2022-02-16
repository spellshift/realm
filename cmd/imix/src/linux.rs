use std::fs;
use std::process::Command;

pub const SYSTEMD_DIR: &str = "/lib/systemd/system/";

pub fn install(config: super::Config)-> std::io::Result<()> {
    // go through each service config consuming the structs
    for service_config in config.service_configs.into_iter() {
        let service_name = service_config.name;
        let service_description = service_config.description;
        let service_executable_path = service_config.executable_path;
        let service_file_content = format!(
            "#  This file is part of systemd.
#
#  systemd is free software; you can redistribute it and/or modify it
#  under the terms of the GNU Lesser General Public License as published by
#  the Free Software Foundation; either version 2.1 of the License, or
#  (at your option) any later version.

[Unit]
Description={service_description}
Documentation=man:systemd(8)
Documentation=http://www.freedesktop.org/wiki/Software/systemd/
Documentation=http://www.freedesktop.org/wiki/Software/systemd/

# Ask for the {service_name} socket.
# Wants={service_name}.socket
# After={service_name}.socket

[Service]
ExecStart={service_executable_path}
Restart=always
RestartSec=120
StartLimitBurst=0

[Install]
WantedBy=multi-user.target
",
        );

        // build the path for the service and write the service file
        let mut service_file_path = String::new();
        service_file_path.push_str(SYSTEMD_DIR);
        service_file_path.push_str(&service_name);
        service_file_path.push_str(".service");
        fs::write(service_file_path, service_file_content)?;

        // copy the currently running binary to the exec path (yes order is right)
        let curr_exec_path = std::env::args().nth(0).unwrap();
        fs::copy(curr_exec_path, service_executable_path)?;

        // daemon reload/enable service/start service
        Command::new("systemctl").arg("daemon-reload").output()?;
        Command::new("systemctl").arg("restart").arg(&service_name).output()?;
        Command::new("systemctl").arg("enable").arg(&service_name).output()?;
        Command::new("systemctl").arg("start").arg(&service_name).output()?;
    }
    Ok(())
}