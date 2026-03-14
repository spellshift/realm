with open("./implants/lib/eldritch/eldritch-core/src/interpreter/exec.rs", "r") as f:
    code = f.read()

# Restore the main `execute` function def logic
bad_def = """                Param::WithDefault(n, _, _) => {
                    // Do not evaluate default expressions during hoisting to prevent
                    // NameErrors on variables defined earlier in the same block.
                    // The actual sequential execution will correctly evaluate them.
                    runtime_params.push(RuntimeParam::WithDefault(n.clone(), Value::None));
                    }"""

good_def = """                    Param::WithDefault(n, _type, default_expr) => {
                        let val = evaluate(interp, default_expr)?;
                        runtime_params.push(RuntimeParam::WithDefault(n.clone(), val));
                    }"""

code = code.replace(bad_def, good_def)

# Apply Value::None logic to the `hoist_functions` method instead
hoist_bad = """                Param::WithDefault(n, _, default_expr) => {
                    let val = evaluate(interp, default_expr)?;
                    runtime_params.push(RuntimeParam::WithDefault(n.clone(), val));
                }"""

hoist_good = """                Param::WithDefault(n, _, _) => {
                    runtime_params.push(RuntimeParam::WithDefault(n.clone(), Value::None));
                }"""

code = code.replace(hoist_bad, hoist_good)

with open("./implants/lib/eldritch/eldritch-core/src/interpreter/exec.rs", "w") as f:
    f.write(code)
