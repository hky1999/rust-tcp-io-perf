extern crate bytes;
extern crate rust_tcp_io_perf;

use rust_tcp_io_perf::connection;
use std::io::Write;
use rust_tcp_io_perf::config;
use rust_tcp_io_perf::threading;


fn main() {
    let args = config::parse_config();

    println!("Connecting to the server {}...", args.address);
    let n_rounds = args.n_rounds;
    let n_bytes = args.n_bytes;
    let tot_n_bytes = (n_bytes * args.n_rounds) as u64;

    println!(
        "client send {} rounds for {} bytes, total {} bytes",
        n_rounds, n_bytes, tot_n_bytes
    );

    if let Ok(mut stream) = connection::client_connect(args.address_and_port()) {
        connection::setup(&args, &stream);
		threading::setup(&args);
        println!("Connection established! Ready to send...");

        // Create a buffer of 0s, size n_bytes, to be sent over multiple times
        let buf = vec![0; n_bytes];
        let progress_tracking_percentage = n_rounds / 10;

        for i in 0..n_rounds {
            match stream.write(&buf) {
                Ok(_n) => {}
                Err(err) => panic!("crazy stuff happened while sending {}", err),
            }
            if i % progress_tracking_percentage == 0 {
                println!("{}0% completed", i / progress_tracking_percentage);
            }
        }
        println!("Sent everything!");

        connection::close_connection(&stream);
    } else {
        println!("Couldn't connect to server...");
    }
}
