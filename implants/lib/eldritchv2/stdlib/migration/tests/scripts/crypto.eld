# crypto.eld
print("=== crypto.md5 ===")
print(crypto.md5(b"hello"))

print("=== crypto.sha1 ===")
print(crypto.sha1(b"hello"))

print("=== crypto.sha256 ===")
print(crypto.sha256(b"hello"))

print("=== crypto.encode_b64 ===")
encoded = crypto.encode_b64("hello")
print(encoded)

print("=== crypto.decode_b64 ===")
decoded = crypto.decode_b64(encoded)
print(decoded)

print("=== crypto.encode_hex ===")
hex_encoded = crypto.encode_hex("hello")
print(hex_encoded)

print("=== crypto.decode_hex ===")
hex_decoded = crypto.decode_hex(hex_encoded)
print(hex_decoded)

print("=== crypto.to_json / from_json ===")
data = {"a": 1, "b": "test"}
json_str = crypto.to_json(data)
# JSON key order might vary, so we parse it back to compare structure
parsed = crypto.from_json(json_str)
print(parsed.get("a"))
print(parsed.get("b"))

print("=== dir(crypto) ===")
print(dir(crypto))
