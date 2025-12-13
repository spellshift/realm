# process.eld
print("=== process.list ===")
procs = process.list()
print(type(procs) == "list")
if len(procs) > 0:
    p = procs[0]
    print(type(p) == "dict")

print("=== process.info ===")
# Use a PID from list if possible, or a fake PID
pid = 123
info = process.info(pid)
print(type(info) == "dict")

print("=== process.name ===")
name = process.name(pid)
print(type(name) == "str")
