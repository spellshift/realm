# file.eld
print("=== file.exists ===")
print(file.exists("/home/user/notes.txt"))
print(file.exists("/home/user/missing.txt"))

print("=== file.read ===")
print(file.read("/home/user/notes.txt"))

print("=== file.write ===")
file.write("/tmp/test.txt", "hello world")
print(file.read("/tmp/test.txt"))

print("=== file.append ===")
file.append("/tmp/test.txt", " appended")
print(file.read("/tmp/test.txt"))

print("=== file.list ===")
# Sort list output to ensure deterministic order for comparison
items = file.list("/home/user")
names = []
for item in items:
    names.append(item.get("file_name"))
names.sort()
print(names)

print("=== file.is_dir / is_file ===")
print(file.is_dir("/home/user"))
print(file.is_file("/home/user/notes.txt"))

print("=== file.mkdir ===")
file.mkdir("/tmp/newdir")
print(file.is_dir("/tmp/newdir"))

print("=== file.copy ===")
file.copy("/home/user/notes.txt", "/tmp/notes_copy.txt")
print(file.read("/tmp/notes_copy.txt"))

print("=== file.move ===")
file.move("/tmp/notes_copy.txt", "/tmp/notes_moved.txt")
print(file.exists("/tmp/notes_copy.txt"))
print(file.read("/tmp/notes_moved.txt"))

print("=== file.remove ===")
file.remove("/tmp/notes_moved.txt")
print(file.exists("/tmp/notes_moved.txt"))

print("=== file.parent_dir ===")
print(file.parent_dir("/home/user/notes.txt"))

print("=== dir(file) ===")
print(dir(file))
