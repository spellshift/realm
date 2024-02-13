pub mod create_remote_thread;
pub mod open_process;
pub mod virtual_alloc_ex;
pub mod write_process_memory;

fn get_u8_vec_form_u32_vec(u32_vec: Vec<u32>) -> anyhow::Result<Vec<u8>> {
    let mut should_err = false;
    let res_u8_vec: Vec<u8> = u32_vec
        .iter()
        .map(|x| {
            if *x <= u8::MAX as u32 {
                *x as u8
            } else {
                should_err = true;
                u8::MAX
            }
        })
        .collect();
    if should_err {
        return Err(anyhow::anyhow!(
            "Error casting eldritch number to u8. Number was too big."
        ));
    }
    Ok(res_u8_vec)
}
