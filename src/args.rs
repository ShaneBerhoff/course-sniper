use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "course-sniper")]
#[command(version = "0.1.0")]
#[command(about = "Precision Registration Tool", long_about = None)]
pub struct SniperArgs {
    /// Runs course-sniper headlessly
    #[arg(short, long)]
    pub detached: bool,

    /// Number of snipers that will run
    #[arg(short, long, value_name = "NUMBER", default_value_t = 1, value_parser = clap::value_parser!(u8).range(1..20))]
    pub snipers: u8,
}
