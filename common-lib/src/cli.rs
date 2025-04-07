use core::str::FromStr;

use heapless::String;
use heapless::Vec;

use crate::prelude::*;

use thiserror::Error;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

pub struct Shell {
    commands: Vec<&'static dyn Command, 50>,
}

#[derive(Error, Debug, PartialEq)]
pub enum ShellError {
    #[error("")]
    IncorrectArgs,
}

pub trait Command {
    fn get_root(&self) -> &'static str;
    fn get_channel(&self) -> &Channel<CriticalSectionRawMutex, String<256>, 5>;
    fn get_sub_commands(&self) -> &[SubCommand];
}

pub struct SubCommand {
    command: &'static str,
    args: usize,
}

pub struct RootCommand<const N: usize> {
    root: &'static str,
    sub: [SubCommand; N],
    channel: Channel<CriticalSectionRawMutex, String<256>, 5>,
}

impl<const N: usize> RootCommand<N> {
    pub const fn new(root: &'static str, sub: [SubCommand; N]) -> Self {
        Self {
            root,
            sub,
            channel: Channel::new(),
        }
    }
}

impl<const N: usize> Command for RootCommand<N> {
    fn get_root(&self) -> &'static str {
        self.root
    }

    fn get_sub_commands(&self) -> &[SubCommand] {
        &self.sub
    }

    fn get_channel(&self) -> &Channel<CriticalSectionRawMutex, String<256>, 5> {
        &self.channel
    }
}

impl Shell {
    fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn register(&mut self, command: &'static dyn Command) -> Receiver {
        let _ = self.commands.push(command);
        Receiver::new(command.get_channel())
    }

    pub async fn send(&mut self, raw_command: String<256>) -> Result<(), ShellError> {
        let split_command: Vec<String<50>, 10> = raw_command
            .split(' ')
            .map(|s| String::from_str(s).unwrap())
            .collect();

        if split_command.is_empty() {
            return Ok(());
        }

        let root_command = split_command[0].as_str();
        let sub_command = split_command.get(1).map_or("", |s| s.as_str());

        for command in &self.commands {
            if command.get_root() == root_command {
                if split_command.len() == 1 {
                    command.get_channel().send(raw_command.clone()).await;
                    return Ok(());
                }
                for subcmd in command.get_sub_commands() {
                    if subcmd.command == sub_command {
                        if subcmd.args + 2 == split_command.len() {
                            command.get_channel().send(raw_command.clone()).await;
                            break;
                        } else {
                            return Err(ShellError::IncorrectArgs);
                        }
                    } else {
                        return Err(ShellError::IncorrectArgs);
                    }
                }
            }
        }

        Ok(())
    }
}

pub struct Receiver {
    channel: &'static Channel<CriticalSectionRawMutex, String<256>, 5>,
}

impl Receiver {
    fn new(receiver: &'static Channel<CriticalSectionRawMutex, String<256>, 5>) -> Receiver {
        Self { channel: receiver }
    }

    pub async fn get(&mut self) -> String<256> {
        self.channel.receive().await
    }
}

#[cfg(test)]
mod test {
    use embassy_time::{Duration, WithTimeout};

    use super::*;

    #[futures_test::test]
    async fn basic_send_and_get() {
        let mut shell = Shell::new();

        static ROOT: RootCommand<0> = RootCommand::new("Hello", []);
        let mut receiver = shell.register(&ROOT);

        let command: String<256> = String::try_from("Hello").unwrap();
        shell.send(command.clone()).await.unwrap();
        let out = receiver
            .get()
            .with_timeout(Duration::from_secs(1))
            .await
            .unwrap();

        assert_eq!(out, command);
    }

    #[futures_test::test]
    async fn basic_send_and_get_queue() {
        let mut shell = Shell::new();

        static ROOT: RootCommand<0> = RootCommand {
            root: "Hello",
            sub: [],
            channel: Channel::new(),
        };

        let mut receiver = shell.register(&ROOT);

        let command: String<256> = String::try_from("Hello").unwrap();
        shell.send(command.clone()).await;
        shell.send(command.clone()).await;

        let out = receiver
            .get()
            .with_timeout(Duration::from_secs(1))
            .await
            .unwrap();

        assert_eq!(out, command);

        let out = receiver
            .get()
            .with_timeout(Duration::from_secs(1))
            .await
            .unwrap();

        assert_eq!(out, command);
    }

    async fn basic_send_and_get_two_receivers() {
        let mut shell = Shell::new();

        static ROOTA: RootCommand<0> = RootCommand {
            root: "Hello",
            sub: [],
            channel: Channel::new(),
        };

        static ROOTB: RootCommand<0> = RootCommand {
            root: "Goodbye",
            sub: [],
            channel: Channel::new(),
        };

        let mut receiver_a = shell.register(&ROOTA);
        let mut receiver_b = shell.register(&ROOTB);

        let command_a: String<256> = String::try_from("Hello").unwrap();
        let command_b: String<256> = String::try_from("Hello").unwrap();

        shell.send(command_a.clone()).await;
        shell.send(command_b.clone()).await;

        let out_a = receiver_a
            .get()
            .with_timeout(Duration::from_secs(1))
            .await
            .unwrap();

        assert_eq!(out_a, command_a);

        let out_b = receiver_b
            .get()
            .with_timeout(Duration::from_secs(1))
            .await
            .unwrap();

        assert_eq!(out_b, command_b);
    }

    #[futures_test::test]
    async fn test_send_and_get_sub_cmd() {
        let mut shell = Shell::new();

        static ROOT: RootCommand<1> = RootCommand {
            root: "Hello",
            sub: [SubCommand {
                command: "world",
                args: 0,
            }],
            channel: Channel::new(),
        };
        let mut rev = shell.register(&ROOT);

        let command: String<256> = String::try_from("Hello world").unwrap();
        shell.send(command.clone()).await;

        let out = rev
            .get()
            .with_timeout(Duration::from_secs(1))
            .await
            .unwrap();

        assert_eq!(out, command);
    }

    #[futures_test::test]
    async fn basic_send_and_get_with_arg() {
        let mut shell = Shell::new();

        static ROOT: RootCommand<1> = RootCommand {
            root: "Hello",
            sub: [SubCommand {
                command: "world",
                args: 1,
            }],
            channel: Channel::new(),
        };

        let mut rev = shell.register(&ROOT);

        let command: String<256> = String::try_from("Hello world 5").unwrap();
        shell.send(command.clone()).await.unwrap();
        let out = rev
            .get()
            .with_timeout(Duration::from_secs(1))
            .await
            .unwrap();

        assert_eq!(out, command);
    }

    #[futures_test::test]
    async fn basic_send_and_get_sub_cmd_with_too_many_args() {
        let mut shell = Shell::new();

        static ROOT: RootCommand<1> = RootCommand {
            root: "Hello",
            sub: [SubCommand {
                command: "world",
                args: 0,
            }],
            channel: Channel::new(),
        };

        let mut rev = shell.register(&ROOT);

        let command: String<256> = String::try_from("Hello world 5").unwrap();
        let out = shell.send(command.clone()).await.unwrap_err();

        assert_eq!(out, ShellError::IncorrectArgs);
    }
}
