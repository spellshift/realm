assets.copy("hello_world.sh","/tmp/abcd")
sys.exec("/bin/sh",["-c","/tmp/abcd"],False)