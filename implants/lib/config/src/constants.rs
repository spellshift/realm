macro_rules! callback_uri {
    () => {
        match option_env!("IMIX_CALLBACK_URI") {
            Some(uri) => uri,
            None => "http://127.0.0.1:8000",
        }
    };
}

/*
 * Compile-time constant for the agent proxy URI, derived from the IMIX_PROXY_URI environment variable during compilation.
 * Defaults to None if this is unset.
 */
macro_rules! proxy_uri {
    () => {
        option_env!("IMIX_PROXY_URI")
    };
}

/*
 * Compile-time constant for the agent callback URI, derived from the IMIX_CALLBACK_URI environment variable during compilation.
 * Defaults to "http://127.0.0.1:8000/grpc" if this is unset.
 */
pub const CALLBACK_URI: &str = callback_uri!();

macro_rules! callback_interval {
    () => {
        match option_env!("IMIX_CALLBACK_INTERVAL") {
            Some(interval) => interval,
            None => "5",
        }
    };
}
/* Compile-time constant for the agent retry interval, derived from the IMIX_RETRY_INTERVAL environment variable during compilation.
 * Defaults to 5 if unset.
 */
pub const CALLBACK_INTERVAL: &str = callback_interval!();

macro_rules! retry_interval {
    () => {
        match option_env!("IMIX_RETRY_INTERVAL") {
            Some(interval) => interval,
            None => "5",
        }
    };
}
/* Compile-time constant for the agent callback interval, derived from the IMIX_CALLBACK_INTERVAL environment variable during compilation.
 * Defaults to 5 if unset.
 */
pub const RETRY_INTERVAL: &str = retry_interval!();

// Compile-time check: CALLBACK_URI must not contain query parameters if RETRY_INTERVAL or CALLBACK_INTERVAL are set
const _: () = {
    const fn contains_query_param(s: &str) -> bool {
        // Can't use normal string ops in const fn.
        let bytes = s.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'?' {
                return true;
            }
            i += 1;
        }
        false
    }

    if contains_query_param(CALLBACK_URI)
        && (option_env!("IMIX_RETRY_INTERVAL").is_some()
            || option_env!("IMIX_CALLBACK_INTERVAL").is_some())
    {
        panic!("CALLBACK_URI cannot contain query parameters when IMIX_RETRY_INTERVAL or IMIX_CALLBACK_INTERVAL environment variables are set");
    }
};

macro_rules! run_once {
    () => {
        match option_env!("IMIX_RUN_ONCE") {
            Some(_) => true,
            None => false,
        }
    };
}

/* Compile-time constant for the agent run once flag, derived from the IMIX_RUN_ONCE environment variable during compilation.
 * Defaults to false if unset.
 */
pub const RUN_ONCE: bool = run_once!();

// Re-export proxy_uri macro for use in system module
pub(crate) use proxy_uri;
