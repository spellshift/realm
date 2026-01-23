# time.eld
print("=== time.now ===")
t = time.now()
print(type(t) == "int")

print("=== time.format_to_readable ===")
# Assuming fake time or specific timestamp for consistency?
# If time.now() is real time, this test is non-deterministic.
# However, for migration testing, we might just check it doesn't crash.
# Or better, use a fixed timestamp.
fixed_time = 1609459200
formatted = time.format_to_readable(fixed_time, "%Y-%m-%d")
print(formatted)

print("=== time.format_to_epoch ===")
epoch = time.format_to_epoch("2021-01-01", "%Y-%m-%d")
# Depending on timezone defaults, this might vary.
# But v1 and v2 should ideally match if defaults are same.
print(type(epoch) == "int")
