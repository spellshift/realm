# sys.eld
print("=== sys.get_env ===")
env = sys.get_env()
print(type(env) == "dict")

print("=== sys.hostname ===")
# Fake bindings usually return "fake-hostname" or similar
print(sys.hostname())

print("=== sys.get_user ===")
user = sys.get_user()
print(type(user) == "dict")
# Check expected keys if possible, but keeping it generic for now

print("=== sys.get_os ===")
os = sys.get_os()
print(type(os) == "dict")

print("=== sys.is_windows / linux / macos ===")
# These should be mutually exclusive or all false depending on fake impl
print(sys.is_windows())
print(sys.is_linux())
print(sys.is_macos())
