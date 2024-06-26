use clap::Parser;
use rand::distributions::{Alphanumeric, DistString};
use ring::digest;
use std::fs::File;
use std::io::Write;

#[derive(Parser)]
#[command()]
struct Args {
    /// The exponent used in calculating the number of hashes to search (base used is 10)
    #[arg(short, long, default_value_t = 8)]
    exponent: u32,

    /// The number of threads to use
    #[arg(short, long, default_value_t = 8)]
    threads: usize,

    /// The prefix length to use
    #[arg(short, long, default_value_t = 12)]
    prefix_len: usize,

    /// The number of updates/thread you want throughout the scan
    #[arg(short, long, default_value_t = 2)]
    updates: usize,

    /// Iterations to do
    #[arg(short, long, default_value_t = 1)]
    iters: usize,
}

fn main() {
    let args = Args::parse();

    let iters = usize::pow(10, args.exponent);
    println!("Checking {iters} hashes across {} threads.", args.threads);

    for _ in 0..args.iters {
        let mut threads = Vec::with_capacity(args.threads);
        let thread_chunk_size = iters / args.threads;

        let start = std::time::Instant::now();

        (0..args.threads).for_each(|thread_id| {
            threads.push(std::thread::spawn(move || {
                let lower_bound = thread_id * thread_chunk_size;
                let upper_bound = (thread_id + 1) * thread_chunk_size;

                let mut rng = rand::thread_rng();
                let prefix = Alphanumeric.sample_string(&mut rng, args.prefix_len);

                let mut input_best = String::from("Initial value.");
                let mut thread_best = digest::digest(&digest::SHA256, input_best.as_bytes());
                let mut checkpoints =
                    rand::seq::index::sample(&mut rng, thread_chunk_size, args.updates).into_vec();

                for point in &mut checkpoints {
                    *point += lower_bound;
                }

                for hash_idx in lower_bound..upper_bound {
                    let input = format!("taxborn/www+taxborn+com+sha/{prefix}/{hash_idx}");
                    let current_guess = digest::digest(&digest::SHA256, input.as_bytes());

                    if checkpoints.contains(&hash_idx) {
                        println!(
                            "Thread {thread_id:>2} current best {thread_best:?} ({input_best}) ({:.3}%)",
                            (hash_idx as f32 - lower_bound as f32) / thread_chunk_size as f32 * 100.0
                        );
                    }

                    if current_guess.as_ref() < thread_best.as_ref() {
                        thread_best = current_guess;
                        input_best = input;
                    }
                }

                (input_best, thread_best)
            }))
        });

        let mut min = digest::digest(&digest::SHA256, b"Initial value.");
        let mut best_input: String = Default::default();

        threads.into_iter().for_each(|thread| {
            let (b_input, thread_min) = thread.join().unwrap();

            if thread_min.as_ref() < min.as_ref() {
                min = thread_min;
                best_input = b_input;
            }
        });

        let elapsed = start.elapsed().as_secs_f32();

        println!("Found {best_input} - {min:?}");
        println!(
            "Averaged {:.3} MHs (took {:.3}s)",
            iters as f32 / elapsed / 1000.0 / 1000.0,
            elapsed
        );

        let mut f = File::options()
            .append(true)
            .open("output.log")
            .expect("Unable to open output.log");
        writeln!(&mut f, "{min:?}:{best_input}").expect("Unable to write to output.log");
    }
}
