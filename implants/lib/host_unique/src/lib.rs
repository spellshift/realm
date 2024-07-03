use uuid::Uuid;

mod env;
pub use env::Env;
mod file;
pub use file::File;

pub trait HostUniqueEngine {
    fn get_name(&self) -> String;
    fn get_host_id(&self) -> Option<Uuid>;
}

// Iterate through all available uniqueness engines in order if one
// returns a UUID that's accepted
pub fn id(engines: Vec<Box<dyn HostUniqueEngine>>) -> Uuid {
    for engine in engines {
        match engine.get_host_id() {
            Some(res) => {
                return res;
            }
            None => {
                #[cfg(debug_assertions)]
                log::debug!("Unique engine {} failed", engine.get_name());
            }
        }
    }

    // All else fails give it a unique one
    Uuid::new_v4()
}

// Return the default list of unique engines to evaluate
// List is evaluated in order and will take the first successful
// result.
pub fn defaults() -> Vec<Box<dyn HostUniqueEngine>> {
    vec![Box::new(Env {}), Box::new(File {})]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_consistent() {
        let engines: Vec<Box<dyn HostUniqueEngine>> = defaults();
        let id_one = id(engines);

        let engines: Vec<Box<dyn HostUniqueEngine>> = defaults();
        let id_two = id(engines);

        assert_eq!(id_one, id_two);
    }

    #[test]
    fn test_id_env_override() {
        std::env::set_var("IMIX_HOST_ID", "f17b92c0-e383-4328-9017-952e5d9fd53d");
        let engines: Vec<Box<dyn HostUniqueEngine>> = defaults();
        let id = id(engines);

        assert_eq!(
            id.to_string(),
            "f17b92c0-e383-4328-9017-952e5d9fd53d".to_string()
        );
    }
}
