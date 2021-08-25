use std::io;

use std::sync::Arc;
use std::thread;

use tokio::sync::Mutex;

use teloxide::prelude::*;
use tokio::sync::mpsc;

use crate::command::execute_command;
use crate::command::Command;
use crate::command::Data;

pub async fn print_bot_info(bot: &AutoSend<Bot>) {
    let bot_info = match bot.get_me().await {
        Ok(me) => me,
        Err(e) => {
            log::error!("Unable to fetch the Bot's Information.{}", e);
            panic!("Unable to fetch the Bot's Information.");
        }
    };
    let bot_username = bot_info.user.username.unwrap();
    log::info!(
        "Bot Name:{} {}\nBot Username:{}",
        bot_info.user.first_name,
        match bot_info.user.last_name {
            Some(s) => s,
            None => String::from(""),
        },
        bot_username
    );
}

use teloxide::utils::command::BotCommand;

pub async fn handle_terminal(data: &Arc<Mutex<Data>>) -> Result<(), std::io::Error> {
    let mut channel = spawn_stdin_channel();
    loop {
        let command = match channel.recv().await {
            Some(c) => {
                if let Command::Stop = c {
                    return Ok(());
                }
                c
            }
            None => {
                continue;
            }
        };
        if let Err(e) = execute_command(data, None, &command, None, true).await {
            log::error!("Command executing error.{}", e);
        }
    }
}

fn spawn_stdin_channel() -> UnboundedReceiver<Command> {
    let (tx, rx) = mpsc::unbounded_channel();
    thread::spawn(move || loop {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();
        let buffer = match Command::parse(buffer.trim(), "") {
            Ok(c) => c,
            Err(e) => {
                log::error!("{}", e);
                continue;
            }
        };
        tx.send(buffer).unwrap();
    });
    rx
}
