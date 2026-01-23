# agent.eld
print("=== agent.get_config ===")
print(agent.get_config())

print("=== agent.get_id ===")
print(agent.get_id())

print("=== agent.get_platform ===")
print(agent.get_platform())

print("=== agent.get_transport ===")
print(agent.get_transport())

print("=== agent.list_transports ===")
print(agent.list_transports())

print("=== agent.get_callback_interval ===")
print(agent.get_callback_interval())

print("=== agent.set_callback_interval ===")
agent.set_callback_interval(60)

print("=== dir(agent) ===")
print(dir(agent))
