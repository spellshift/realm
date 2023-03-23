assets.copy("exec_script/hello_word.sh","/tmp/abcd")
print(sys.shell("chmod +x /tmp/abcd && /tmp/abcd"))