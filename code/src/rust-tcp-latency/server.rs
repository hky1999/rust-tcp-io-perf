extern crate bytes;
extern crate rust_tcp_io_perf;

use rust_tcp_io_perf::config;
use rust_tcp_io_perf::connection;
use rust_tcp_io_perf::threading;

fn main() {
    let args = config::parse_config();
    let n_bytes = args.n_bytes;
    let n_rounds = args.n_rounds;
    let mut buf = vec![0; n_bytes];

    println!(
        "server start: n_bytes {}, n_rounds {}, port {}",
        n_bytes, n_rounds, args.port
    );

    let mut stream = connection::server_listen_and_get_first_connection(&args.port);
    connection::setup(&args, &mut stream);
    threading::setup(&args);

    // Make sure n_rounds is the same between client and server
    for _i in 0..(n_rounds * 2) {
        connection::receive_message(n_bytes, &mut stream, &mut buf);
        connection::send_message(n_bytes, &mut stream, &buf);
    }

    println!("Done exchanging stuff")
}
