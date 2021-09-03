use std::time::Duration;

use kdmapi::KDMAPI;

fn main() {
    let kdmapi = KDMAPI.open_stream();

    kdmapi.send_direct_data(0x7F4090);

    std::thread::sleep(Duration::from_secs(5));

    // kdmapi dropped, terminating the stream here
}
