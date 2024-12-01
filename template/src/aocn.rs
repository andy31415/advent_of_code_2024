use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[tracing::instrument]
fn main() {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    let stdout_log = tracing_subscriber::fmt::layer().compact();
    
    tracing_subscriber::registry()
        .with(stdout_log)
        .with(EnvFilter::from_default_env())
        .init();

    let s1 = {{project-name}}::part1(include_str!("../input.txt"));
    println!("Part 1: {}", s1);

    let s2 = {{project-name}}::part2(include_str!("../input.txt"));
    println!("Part 2: {}", s2);
}
