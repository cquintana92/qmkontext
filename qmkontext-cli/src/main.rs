#[macro_use]
extern crate tracing;

mod conf;
mod list;
mod utils;

use clap::{Parser, Subcommand};
use qmkontext::{
    chrono::Duration, CliSink, Engine, HidEventSink, UserEventConfig, UserEventSource,
    UserEventSourceKind,
};
use std::collections::HashMap;

const RETRY_DELAY_SECONDS: u64 = 10;

#[derive(Parser)]
#[command(name = "QMKontext")]
#[command(author, version)]
struct Args {
    #[arg(short, long)]
    config: Option<String>,
    #[command(subcommand)]
    command: Option<Subcommands>,
}

#[derive(Subcommand)]
enum Subcommands {
    List,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let config = conf::Config::new(args.config).expect("Error reading config");

    utils::setup_logging(&config.log_level);

    if let Some(Subcommands::List) = args.command {
        if let Err(e) = list::list_hid_devices() {
            error!("Error listing devices: {:?}", e);
        }
        return Ok(());
    }

    let mut configs: Vec<UserEventConfig> = Vec::new();
    if config.current_program.enable {
        let mut mappings = HashMap::new();
        for mapping in config.current_program.mappings {
            mappings.insert(mapping.key, mapping.value);
        }

        configs.push(UserEventConfig {
            interval: Duration::seconds(config.current_program.interval_seconds as i64),
            kind: UserEventSourceKind::CurrentProgram {
                mappings,
                default_value: config.current_program.default_value,
                use_lowercase: config.current_program.use_lowercase,
            },
            command_id: config.current_program.command_id,
        })
    }

    for custom_command in config.custom_commands {
        configs.push(UserEventConfig {
            interval: Duration::seconds(custom_command.interval_seconds as i64),
            kind: UserEventSourceKind::UserDefined {
                command: custom_command.command,
            },
            command_id: custom_command.command_id,
        })
    }

    let source = UserEventSource::new(configs, 10);
    if config.debug_mode {
        let engine = Engine::new(source, CliSink);
        engine.start().expect("Error in loop");
    } else {
        let keyboard = config.keyboard;
        let sink = loop {
            match HidEventSink::new(
                keyboard.vendor_id,
                keyboard.product_id,
                keyboard.usage,
                keyboard.usage_page,
            ) {
                Ok(c) => break c,
                Err(e) => {
                    error!("Cannot connect to device: {:?}", e);
                    std::thread::sleep(std::time::Duration::from_secs(RETRY_DELAY_SECONDS));
                    continue;
                }
            }
        };
        let engine = Engine::new(source, sink);
        engine.start().expect("Error in loop")
    };

    Ok(())
}
