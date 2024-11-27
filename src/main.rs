use clap::{value_parser, Arg, Command};
use ftsov2mon::commands::start;
use tracing::error;

#[tokio::main(flavor = "multi_thread", worker_threads = 5)]
async fn main() -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .with_line_number(false)
        .with_target(true)
        .with_ansi(true)
        .with_level(true)
        .with_thread_ids(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    // Comes from the build script
    let version_str = include_str!(concat!(env!("OUT_DIR"), "/version_file"));

    let matches = Command::new("ftsov2mon")
        .about("Flare FTSOv2 Monitoring Tool and Metrics Exporter")
        .author("Suryansh @ Luganodes")
        .version(version_str)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("start")
                .about("Start monitoring")
                .arg_required_else_help(false)
                .args([
                    Arg::new("tg-api-key")
                        .long("tg-api-key")
                        .requires("tg-chat-id")
                        .default_value(""),
                    Arg::new("tg-chat-id")
                        .long("tg-chat-id")
                        .requires("tg-api-key")
                        .default_value(""),
                    Arg::new("metrics-port")
                        .long("metrics-port")
                        .value_parser(value_parser!(u16))
                        .default_value("6969"),
                    Arg::new("metrics-addr")
                        .long("metrics-addr")
                        .default_value("0.0.0.0"),
                    Arg::new("rpc-url")
                        .long("rpc-url")
                        .help("A Flare Network JSON RPC URL")
                        .required(true),
                    Arg::new("block-window")
                        .long("block-window")
                        .value_parser(value_parser!(u16))
                        .default_value("100")
                        .help("The number of blocks from now in the past to monitor"),
                    Arg::new("submit-address")
                        .long("submit-address")
                        .alias("sa")
                        .help("The FTSO Submit Address")
                        .required(true),
                    Arg::new("submit-signature-address")
                        .long("submit-signature-address")
                        .alias("ssa")
                        .help("The FTSO Submit Signature Address")
                        .required(true),
                    Arg::new("signing-policy-address")
                        .long("signing-policy-address")
                        .alias("spa")
                        .help("The FTSO Signing Policy Address")
                        .required(true),
                ]),
        )
        .get_matches();

    let res = match matches.subcommand() {
        Some(("start", sub_m)) => start(sub_m).await,
        None | Some(_) => unreachable!(),
    };

    match res {
        Ok(_) => {}
        Err(err) => {
            error!("Error: {err:?}");
        }
    }

    Ok(())
}
