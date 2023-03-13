extern crate bytes;
extern crate rust_tcp_io_perf;
extern crate hdrhist;
extern crate meansd;

use meansd::MeanSD;
use std::time::Instant;
use std::{thread, time};
use rust_tcp_io_perf::config;
use rust_tcp_io_perf::connection;
use rust_tcp_io_perf::print_utils;
use rust_tcp_io_perf::threading;

fn main() {
    let args = config::parse_config();

    println!("Connecting to the server {}...", args.address);
    let n_rounds = args.n_rounds;
    let n_bytes = args.n_bytes;

    // Create buffers to read/write
    let wbuf: Vec<u8> = vec![0; n_bytes];
    let mut rbuf: Vec<u8> = vec![0; n_bytes];

    let progress_tracking_percentage = (n_rounds * 2) / 100;

    let mut meansd = MeanSD::default();

    let mut connected = false;

    while !connected {
        match connection::client_connect(args.address_and_port()) {
            Ok(mut stream) => {
                connection::setup(&args, &mut stream);
                threading::setup(&args);
                connected = true;
                let mut hist = hdrhist::HDRHist::new();
                let mut max = u64::MIN;
                let mut min = u64::MAX;
                let mut results = vec![];

                println!("Connection established! Ready to send...");

                // To avoid TCP slowstart we do double iterations and measure only the second half
                for i in 0..(n_rounds * 2) {
                    let start = Instant::now();

                    connection::send_message(n_bytes, &mut stream, &wbuf);
                    connection::receive_message(n_bytes, &mut stream, &mut rbuf);

                    let duration = Instant::now().duration_since(start);
                    let duration_nanoseconds =
                        duration.as_secs() * 1_000_000_000u64 + duration.subsec_nanos() as u64;

                    if i >= n_rounds {
                        hist.add_value(
                            duration.as_secs() * 1_000_000_000u64 + duration.subsec_nanos() as u64,
                        );
                        results.push(duration);
                        meansd.update(duration_nanoseconds as f64 / 1000.0);
                        if duration_nanoseconds > max {
                            max = duration_nanoseconds;
                        }
                        if duration_nanoseconds < min {
                            min = duration_nanoseconds;
                        }
                    }

                    if i % progress_tracking_percentage == 0 {
                        // Track progress on screen
                        println!("{}% completed", i / progress_tracking_percentage);
                    }
                }
                print_utils::print_line();
                println!(
                    "latency max: {}us, min: {}us",
                    max as f64 / 1000.0,
                    min as f64 / 1000.0
                );
                print_utils::print_line();
                println!(
                    "meansd size {} average {}, sstdev {}",
                    meansd.size(),
                    meansd.mean(),
                    meansd.sstdev(),
                );

                let mut sum = 0.0;
                for res in results.clone() {
                    let duration_nanoseconds =
                        res.as_secs() * 1_000_000_000u64 + res.subsec_nanos() as u64;
                    sum = sum + duration_nanoseconds as f64 / 1000.0;
                }

                let len = results.len();
                println!("results size {} average {}", len, sum / len as f64);
                // println!("results: {:?}", results);
                connection::close_connection(&stream);
                // print_utils::print_summary(hist);
            }
            Err(error) => {
                println!("Couldn't connect to server, retrying... Error {}", error);
                thread::sleep(time::Duration::from_secs(1));
            }
        }
    }
}
