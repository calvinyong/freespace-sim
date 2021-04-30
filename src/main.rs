use clap::{App, AppSettings, Arg, SubCommand};
use freespace_sim::prelude::*;

fn print_results(results_vec: Vec<Results>) {
    let len = results_vec.len();
    println!(
        "Average malloc fails: {}",
        results_vec.iter().map(|r| r.malloc_fails).sum::<usize>() as f32 / len as f32
    );
    println!(
        "Average free fails: {}",
        results_vec.iter().map(|r| r.free_fails).sum::<usize>() as f32 / len as f32
    );
    println!(
        "Average internal fragmentation: {}",
        results_vec.iter().map(|r| r.internal_frag).sum::<usize>() as f32 / len as f32
    );
    println!(
        "Average external fragmentation: {}",
        results_vec.iter().map(|r| r.external_frag).sum::<f32>() / len as f32
    );
}

// Benches take the average of 5 runs
fn bench_random(ratio: f64) {
    let num_runs = 5;
    let freelist = FreeList::new(0, 32768, true)
        .align(32)
        .policy(Policy::First);
    let mut results_vec = Vec::new();
    for _ in 0..num_runs {
        results_vec.push(workloads::random_memory(freelist.clone(), ratio));
    }

    println!("Random size allocation with {}% malloc\n", ratio * 100.0);
    println!("Free list results");
    print_results(results_vec);
    println!();

    let buddy = BuddyAllocator::new(5, 15);
    let mut results_vec = Vec::new();
    for _ in 0..num_runs {
        results_vec.push(workloads::random_memory(buddy.clone(), ratio));
    }

    println!("Buddy allocator results");
    print_results(results_vec);
}

fn bench_stack(ratio: f64) {
    let num_runs = 5;
    let freelist = FreeList::new(0, 32768, true)
        .align(32)
        .policy(Policy::First);
    let mut results_vec = Vec::new();
    for _ in 0..num_runs {
        results_vec.push(workloads::stack(freelist.clone(), ratio));
    }

    println!("Fixed size allocation with {}% malloc\n", ratio * 100.0);
    println!("Free list results");
    print_results(results_vec);
    println!();

    let buddy = BuddyAllocator::new(5, 15);
    let mut results_vec = Vec::new();
    for _ in 0..num_runs {
        results_vec.push(workloads::stack(buddy.clone(), ratio));
    }

    println!("Buddy allocator results");
    print_results(results_vec);
}

fn main() {
    let matches = App::new("Free space simulator")
        .author("Calvin")
        .about("Simulates a list based freelist and buddy allocator")
        .version("0.1.0")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("demo")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .about("Demo an allocator")
                .subcommand(
                    SubCommand::with_name("freelist")
                        .about("Run the freelist")
                        .arg(
                            Arg::with_name("coalesce")
                                .long("coalesce")
                                .short("c")
                                .help("Enable coalescing"),
                        ),
                )
                .subcommand(SubCommand::with_name("buddy").about("Run the buddy allocator")),
        )
        .subcommand(
            SubCommand::with_name("bench")
                .about("Run a workload on freelist and buddy")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(
                    SubCommand::with_name("stack")
                        .about("A workload that resembles a stack")
                        .arg(
                            Arg::with_name("ratio")
                                .long("ratio")
                                .short("r")
                                .default_value("0.5")
                                .takes_value(true),
                        ),
                )
                .subcommand(
                    SubCommand::with_name("random")
                        .about("A workload that mallocs random amount of memory")
                        .arg(
                            Arg::with_name("ratio")
                                .long("ratio")
                                .short("r")
                                .default_value("0.5")
                                .takes_value(true),
                        ),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        ("demo", Some(demo)) => match demo.subcommand() {
            ("freelist", Some(freelist)) => demos::freelist(freelist.is_present("coalesce")),
            ("buddy", Some(_)) => demos::buddy(),
            _ => unreachable!(),
        },
        ("bench", Some(bench)) => match bench.subcommand() {
            ("random", Some(random)) => bench_random(
                random
                    .value_of("ratio")
                    .unwrap()
                    .parse()
                    .expect("Could not parse input"),
            ),
            ("stack", Some(stack)) => bench_stack(
                stack
                    .value_of("ratio")
                    .unwrap()
                    .parse()
                    .expect("Could not parse input"),
            ),
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}
