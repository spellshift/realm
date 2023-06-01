def main(target_cidrs):
    print("Scanning {}".format(target_cidrs))
    target_ports = [x for x in range(0,10000)]
    for found_port in pivot.port_scan(target_cidrs, target_ports, "tcp", 2):
        if found_port['status'] == "open":
            print(found_port)
            print(pivot.ncat(found_port['ip'], found_port['port'], "nomnom\nid", "tcp"))

main(["10.10.0.129/32"])