# regex.eld
print("=== regex.match ===")
print(regex.match("hello world", "world"))
print(regex.match("hello world", "universe"))

print("=== regex.match_all ===")
# Depending on implementation, might return list of matches
print(regex.match_all("hello world hello", "hello"))

print("=== regex.replace ===")
print(regex.replace("hello world", "world", "universe"))

print("=== regex.replace_all ===")
print(regex.replace_all("hello world hello", "hello", "hi"))
