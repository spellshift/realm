use std::sync::mpsc::Receiver;
use std::time::Duration;

/*
 * Drain a receiver, returning all currently available results as a Vec.
 */
pub fn drain<T>(reciever: &Receiver<T>) -> Vec<T> {
    let mut result: Vec<T> = Vec::new();
    loop {
        let val = match reciever.recv_timeout(Duration::from_millis(100)) {
            Ok(v) => v,
            Err(err) => {
                match err.to_string().as_str() {
                    "channel is empty and sending half is closed" => {
                        break;
                    }
                    "timed out waiting on channel" => {
                        break;
                    }
                    _ => {
                        #[cfg(debug_assertions)]
                        eprint!("failed to drain channel: {}", err)
                    }
                }
                break;
            }
        };
        result.push(val);
    }
    result
}
