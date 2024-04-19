use crate::{Error, Event, Result};
use chrono::Duration;
use crossbeam_channel::{Receiver, Sender};
use std::collections::HashMap;
use std::process::{Command, Output};

pub trait EventSource {
    fn events(&self) -> Receiver<Event>;
    fn start(self);
}

#[derive(Clone)]
pub enum UserEventSourceKind {
    CurrentProgram {
        mappings: HashMap<String, u8>,
        default_value: u8,
        use_lowercase: bool,
    },
    UserDefined {
        command: String,
    },
}

#[derive(Clone)]
pub struct UserEventConfig {
    pub interval: Duration,
    pub kind: UserEventSourceKind,
    pub command_id: u8,
}

#[derive(Clone)]
pub struct UserEventSource {
    sources: Vec<UserEventConfig>,
    sender: Sender<Event>,
    receiver: Receiver<Event>,
}

struct ActiveProgramData {
    binary: String,
    name: String,
}

impl UserEventSource {
    pub fn new(sources: Vec<UserEventConfig>, buffer_size: usize) -> Self {
        let (sender, receiver) = crossbeam_channel::bounded(buffer_size);
        Self {
            sources,
            sender,
            receiver,
        }
    }

    fn loop_source(source: UserEventConfig, sender: Sender<Event>) {
        let kind = source.kind.clone();
        match kind {
            UserEventSourceKind::CurrentProgram {
                mappings,
                default_value,
                use_lowercase,
            } => Self::loop_current_program(mappings, default_value, use_lowercase, source, sender),
            UserEventSourceKind::UserDefined { command } => {
                Self::loop_user_defined(command, source, sender)
            }
        }
    }
}

impl UserEventSource {
    fn loop_current_program(
        mappings: HashMap<String, u8>,
        default_value: u8,
        use_lowercase: bool,
        source: UserEventConfig,
        sender: Sender<Event>,
    ) {
        loop {
            if let Err(e) = Self::step_current_program(
                &mappings,
                default_value,
                use_lowercase,
                &source,
                &sender,
            ) {
                error!("error in current_program : {:?}", e);
            }

            std::thread::sleep(source.interval.to_std().unwrap())
        }
    }

    fn step_current_program(
        mappings: &HashMap<String, u8>,
        default_value: u8,
        use_lowercase: bool,
        source: &UserEventConfig,
        sender: &Sender<Event>,
    ) -> Result<()> {
        let current_program_data = Self::get_active_program_data()?;

        for (program_name, command_data) in mappings {
            let (current_program_binary, current_program_name, transformed_program_name) =
                if use_lowercase {
                    (
                        current_program_data.binary.to_lowercase(),
                        current_program_data.name.to_lowercase(),
                        program_name.to_lowercase(),
                    )
                } else {
                    (
                        current_program_data.binary.to_string(),
                        current_program_data.name.to_string(),
                        program_name.to_string(),
                    )
                };
            debug!("Program Binary: {}", current_program_binary);
            debug!("Program Name: {}", current_program_name);
            if current_program_name.contains(&transformed_program_name) {
                debug!(
                    "Found program name in name (program_name={})",
                    transformed_program_name
                );
                info!("Found program {program_name}");
                let event = Event::Send {
                    command_id: source.command_id,
                    command_data: *command_data,
                };
                let _ = sender.send(event);
                return Ok(());
            }

            if current_program_binary.contains(&transformed_program_name) {
                debug!(
                    "Found program name in binary (program_name={})",
                    transformed_program_name
                );
                info!("Found program {program_name}");
                let event = Event::Send {
                    command_id: source.command_id,
                    command_data: *command_data,
                };
                let _ = sender.send(event);
                return Ok(());
            }
        }

        debug!("Did not find the current program in mappings, sending default value");
        let default_event = Event::Send {
            command_id: source.command_id,
            command_data: default_value,
        };
        let _ = sender.send(default_event);
        Ok(())
    }

    fn get_active_program_data() -> Result<ActiveProgramData> {
        let window_pid = Self::run_xdotool_command("getwindowpid");
        if !window_pid.status.success() {
            let stderr = Self::stderr_to_string(window_pid)?;
            warn!("Get window pid status was not success: {stderr}");
            return Err(Error::CannotGetCurrentProgram);
        }

        let pid = Self::output_to_string(window_pid)?;
        let pid = pid.parse::<usize>().map_err(|e| {
            warn!("Error parsing pid into usize: pid={}: {:?}", pid, e);
            Error::CannotGetCurrentProgram
        })?;

        let program_commandline = Self::get_program_commandline(pid)?;
        let cmdline_parts = program_commandline.split(' ').collect::<Vec<&str>>();
        let binary = cmdline_parts.first().unwrap();

        let window_name = Self::run_xdotool_command("getwindowname");
        if !window_name.status.success() {
            let stderr = Self::stderr_to_string(window_name)?;
            warn!("Get window name status was not success: {stderr}");
            return Err(Error::CannotGetCurrentProgram);
        }

        let window_name = Self::output_to_string(window_name)?;
        Ok(ActiveProgramData {
            binary: binary.to_string(),
            name: window_name,
        })
    }

    fn output_to_string(output: Output) -> Result<String> {
        let value = String::from_utf8(output.stdout).map_err(|e| {
            warn!("Error converting command output to string: {:?}", e);
            Error::CannotGetCurrentProgram
        })?;

        Ok(value.trim().replace('\n', ""))
    }

    fn stderr_to_string(output: Output) -> Result<String> {
        let value = String::from_utf8(output.stderr).map_err(|e| {
            warn!("Error converting command output to string: {:?}", e);
            Error::CannotGetCurrentProgram
        })?;

        Ok(value.trim().replace('\n', ""))
    }

    fn run_xdotool_command(command: &str) -> Output {
        let cmd = format!("xdotool getwindowfocus {command}");
        Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
            .unwrap_or_else(|_| panic!("Failed to execute 'xdotool getwindowfocus {command}'"))
    }

    fn get_program_commandline(pid: usize) -> Result<String> {
        let cmd = format!("/bin/strings /proc/{pid}/cmdline");
        let output = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
            .unwrap_or_else(|_| panic!("Failed read /proc/{pid}/cmdline'"));
        if !output.status.success() {
            return Err(Error::CannotGetCurrentProgram);
        }

        Self::output_to_string(output)
    }
}

impl UserEventSource {
    fn loop_user_defined(command: String, source: UserEventConfig, sender: Sender<Event>) {
        loop {
            if let Err(e) = Self::step_user_defined(&command, &source, &sender) {
                error!("error in user defined [command={}]: {:?}", command, e);
            }

            std::thread::sleep(source.interval.to_std().unwrap())
        }
    }

    fn step_user_defined(
        command: &str,
        source: &UserEventConfig,
        sender: &Sender<Event>,
    ) -> Result<()> {
        let output = Command::new("/bin/bash")
            .arg("-c")
            .arg(command)
            .output()
            .unwrap_or_else(|_| panic!("Failed to execute custom command [{command}]"));
        let value = Self::output_to_string(output)?;

        let output_number = value.parse::<u8>().map_err(|e| {
            Error::UserConfigExecutionError(format!(
                "Error parsing output into u8: output={}: {:?}",
                value, e
            ))
        })?;

        let event = Event::Send {
            command_id: source.command_id,
            command_data: output_number,
        };
        let _ = sender.send(event);
        Ok(())
    }
}

impl EventSource for UserEventSource {
    fn events(&self) -> Receiver<Event> {
        self.receiver.clone()
    }

    fn start(self) {
        for source in self.sources {
            let sender = self.sender.clone();
            std::thread::spawn(move || {
                Self::loop_source(source, sender);
            });
        }
    }
}
