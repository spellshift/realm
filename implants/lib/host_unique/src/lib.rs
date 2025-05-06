use uuid::Uuid;

mod env;
pub use env::Env;
mod file;
pub use file::File;
mod registry;
pub use registry::Registry;

pub trait HostIDSelector {
    fn get_name(&self) -> String;
    fn get_host_id(&self) -> Option<Uuid>;
}

// Iterate through all available uniqueness selectors in order if one
// returns a UUID that's accepted
pub fn get_id_with_selectors(selectors: Vec<Box<dyn HostIDSelector>>) -> Uuid {
    for selector in selectors {
        match selector.get_host_id() {
            Some(res) => {
                return res;
            }
            None => {
                #[cfg(debug_assertions)]
                log::debug!("Unique selector {} failed", selector.get_name());
            }
        }
    }

    // All else fails give it a unique one
    Uuid::new_v4()
}

// Return the default list of unique selectors to evaluate
// List is evaluated in order and will take the first successful
// result.
pub fn defaults() -> Vec<Box<dyn HostIDSelector>> {
    vec![Box::<Env>::default(), Box::<File>::default()]
}
