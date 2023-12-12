def copy_script_and_execute():
    if sys.is_linux() or sys.is_macos():
        assets.copy("exec_script/hello_world.sh","/tmp/golem_cli_test-copy_script_and_execute")
        shell_res = sys.shell("chmod +x /tmp/golem_cli_test-copy_script_and_execute && /tmp/golem_cli_test-copy_script_and_execute")
        print(shell_res)
        print("\n")
    elif sys.is_windows():
        assets.copy("exec_script/hello_world.bat","C:\Windows\Temp\golem_cli_test-copy_script_and_execute")
        shell_res = sys.shell("C:\Windows\Temp\golem_cli_test-copy_script_and_execute")
        print(shell_res)
        print("\n")

copy_script_and_execute()