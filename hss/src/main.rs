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

/// A pair that keeps track of an input and its resulting SHA256 hash
struct HashPair {
    /// The given input for the hash
    input: String,
    /// The resulting SHA256 hash
    hash: digest::Digest,
}

impl HashPair {
    fn new(input: String) -> Self {
        let hash = digest::digest(&digest::SHA256, input.as_bytes());

        Self { input, hash }
    }

    /// Update the current HashPair to another given input
    fn update(&mut self, input: String) {
        self.input = input.clone();
        self.hash = digest::digest(&digest::SHA256, input.as_bytes());
    }
}

fn main() {
    let args = Args::parse();

    // Calculate the number of hashes
    let iters = usize::pow(10, args.exponent);
    println!(
        "Checking {iters:.1$e} hashes across {} threads.",
        args.threads, 1
    );

    // This is the main loop of work, we repeat it by the number of iterations specified by the
    // user (or default of 1)
    for _ in 0..args.iters {
        // Preallocate a vector of threads
        let mut threads = Vec::with_capacity(args.threads);
        // Get the number of hashes this each thread must compute
        // TODO: This results in some threads getting done earlier than others, where they could
        // also be taking work. If we instead make a shared counter that counts up/down until it
        // reaches `iters`, that might remove the need for this and the bound computations, and
        // even out work as long as there is work to be done. Not sure the speed up this would get
        // but I think it would be measurable, especially due to 'fast' threads being able to keep
        // doing work.
        let thread_hash_share = iters / args.threads;

        let start = std::time::Instant::now();

        // Generate the threads
        (0..args.threads).for_each(|thread_id| {
            threads.push(std::thread::spawn(move || {
                // Compute the share of hashes this thread searches.
                let lower_bound = thread_id * thread_hash_share;
                let upper_bound = (thread_id + 1) * thread_hash_share;

                // Generate a random prefix to make sure each run is different.
                let mut rng = rand::thread_rng();
                let prefix = Alphanumeric.sample_string(&mut rng, args.prefix_len);

                // Keep track of the current best hash and its respective input
                let mut best_result = HashPair::new("Initial value".to_string());

                // Calculate some checkpoints to print out debug information. Not necessarily
                // needed but doesn't seem to impact performance for some nice information.
                let mut debug_checkpoints =
                    rand::seq::index::sample(&mut rng, thread_hash_share, args.updates).into_vec();
                for checkpoint in &mut debug_checkpoints {
                    *checkpoint += lower_bound;
                }

                // Main loop to compute hashes
                for iteration in lower_bound..upper_bound {
                    let guess = format!("taxborn/www+taxborn+com+sha/{prefix}/{iteration}");
                    let hash = digest::digest(&digest::SHA256, guess.as_bytes());

                    if debug_checkpoints.contains(&iteration) {
                        // Since the
                        let progress =
                            (iteration - lower_bound) as f32 * 100.0 / thread_hash_share as f32;

                        println!(
                            "[th-{thread_id:0>2}]-{:?} ({:0>50}) ({progress:.3}% done)",
                            best_result.hash, best_result.input
                        );
                    }

                    // Check to see if we have a new winner
                    if hash.as_ref() < best_result.hash.as_ref() {
                        best_result.update(guess);
                    }
                }

                // Return these values to
                best_result
            }))
        });

        // Record the best hash from all of the threads
        let best_result = threads
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .min_by_key(|hashpair| hashpair.hash.as_ref().to_vec())
            .unwrap();

        // Print out some final statistics
        let elapsed = start.elapsed().as_secs_f32();

        println!(" FOUND: {:?} ({})", best_result.hash, best_result.input);
        println!(
            "Averaged {:.3} MHs (took {:.3}s)",
            iters as f32 / elapsed / 1000.0 / 1000.0,
            elapsed
        );

        // Write out to a file so we can remember these results on subsequent runs
        write_result(best_result);
    }
}

fn write_result(best: HashPair) {
    let mut f = File::options()
        .append(true)
        .open("output.log")
        .expect("Unable to open output.log");
    writeln!(&mut f, "{:?}:{}", best.hash, best.input).expect("Unable to write to output.log");
}
