use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Args {
    #[structopt(short="c", long="config")]
    pub config: String,
    #[structopt(short="v", parse(from_occurrences))]
    pub verbose: usize,
}
