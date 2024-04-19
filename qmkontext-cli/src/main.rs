#[macro_use]
extern crate tracing;

mod conf;
mod list;
mod utils;

use crate::conf::{Config, KeyboardConfig};
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

fn get_sink(keyboards: &[KeyboardConfig]) -> Option<HidEventSink> {
    for keyboard in keyboards.iter() {
        match HidEventSink::new(
            keyboard.vendor_id,
            keyboard.product_id,
            keyboard.usage,
            keyboard.usage_page,
        ) {
            Ok(c) => {
                info!(
                    "Connected to device: vendorid={} productid={}",
                    keyboard.vendor_id, keyboard.product_id
                );
                return Some(c);
            }
            Err(e) => {
                error!("Cannot connect to device: {:?}", e);
            }
        };
    }

    None
}

fn start(
    source: UserEventSource,
    keyboard: Option<KeyboardConfig>,
    keyboards: Vec<KeyboardConfig>,
) {
    let mut configured_keyboards = Vec::new();
    if let Some(k) = keyboard {
        configured_keyboards.push(k);
    } else {
        for keyboard in keyboards {
            configured_keyboards.push(keyboard);
        }
    }

    if configured_keyboards.is_empty() {
        panic!("There are no configured keyboards. Please check your config");
    }

    loop {
        let sink = get_sink(&configured_keyboards);
        match sink {
            Some(s) => {
                let engine = Engine::new(source.clone(), s);
                if let Err(e) = engine.start() {
                    warn!("Error in engine: {:?}", e);
                }
            }
            None => {
                info!("Cannot connect to any keyboard");
                std::thread::sleep(std::time::Duration::from_secs(RETRY_DELAY_SECONDS));
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(RETRY_DELAY_SECONDS))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let config = Config::new(args.config).expect("Error reading config");

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
        start(source, config.keyboard, config.keyboards);
    };

    Ok(())
}
