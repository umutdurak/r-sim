#[no_mangle]
pub extern "C" fn do_step(current_time: f64) -> f64 {
    // Simple FMU-like behavior: output is current_time + 1.0
    current_time + 1.0
}
