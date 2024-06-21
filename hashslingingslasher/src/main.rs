use clap::Parser;
use sha256::digest;
use std::time::Instant;

#[derive(Parser)]
#[command()]
struct Args {
    /// The username for the competition
    #[arg(short, long, default_value_t = String::from("taxborn"))]
    username: String,

    /// Any prefix you want (ex taxborn/{prefix}/32789)
    #[arg(short, long)]
    prefix: Option<String>,

    /// The exponent used to determine how many hashes to search
    /// 10^x where x is the value you provide here.
    #[arg(short, long, default_value_t = 4)]
    exponent: u32,
}

fn main() {
    let args = Args::parse();
    let mut best: String = digest(format!("{}/0", args.username));
    let iters = 10_usize.pow(args.exponent);

    let prefix = if let Some(prefix) = args.prefix {
        format!("{}/{}", args.username, prefix)
    } else {
        args.username
    };

    println!("Starting search with the prefix {prefix} for {iters} hashes");
    println!("Starting value: {best}");

    let now = Instant::now();
    let mut last = Instant::now();

    for i in 0..iters {
        let input = format!("{prefix}/{i}");
        let current_hash = digest(&input);

        if current_hash.lt(&best) {
            let time_since_last = last.elapsed();
            last = Instant::now();
            best = current_hash;
            println!(
                "New best: {best} from ({input}) ({:.2}s since last)",
                time_since_last.as_secs_f64()
            );
            println!("  {:.2}% done with search", i as f32 / iters as f32 * 100.0);
        }
    }

    let leading_zeroes = best.chars().take_while(|c| *c == '0').count();

    let elapsed = now.elapsed();
    println!(
        "Found {best} ({leading_zeroes} 0's) after {:.2}s",
        elapsed.as_secs_f64()
    );
    println!(
        "Average {:.2} MH/s",
        iters as f64 / elapsed.as_secs_f64() / 1000.0 / 1000.0
    );
}
