def get_proc_list():
    for proc in process.list():
        print("[{pid}]\t{username}\t{command}".format(pid=proc['pid'],username=proc['username'],command=proc['command']))

get_proc_list()