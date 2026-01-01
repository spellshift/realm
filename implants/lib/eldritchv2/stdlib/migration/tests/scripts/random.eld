# random.eld
print("=== random.int ===")
# Testing randomness is hard, but we can check types and ranges
val = random.int(1, 10)
print(type(val) == "int")
print(val >= 1 and val <= 10)

print("=== random.string ===")
s = random.string(5)
print(len(s))
print(type(s) == "str")

print("=== random.uuid ===")
u = random.uuid()
print(len(u) > 0)
print(type(u) == "str")

print("=== random.bool ===")
b = random.bool()
print(type(b) == "bool")

print("=== random.bytes ===")
by = random.bytes(4)
print(len(by))
print(type(by) == "bytes")
