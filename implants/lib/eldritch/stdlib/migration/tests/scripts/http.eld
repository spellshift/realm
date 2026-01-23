# http.eld
print("=== http.get ===")
# Fake bindings usually mock specific URLs or return empty
print(http.get("http://example.com"))

print("=== http.post ===")
print(http.post("http://example.com", b"data"))

print("=== http.download ===")
http.download("http://example.com/file", "/tmp/downloaded")
print(file.exists("/tmp/downloaded"))

print("=== dir(http) ===")
print(dir(http))
