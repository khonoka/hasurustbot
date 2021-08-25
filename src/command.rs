use crate::file_data::FileData;

pub struct Data {
    pub admins: FileData,
    pub quotes: FileData,
    pub is_admin: HashSet<i64>,
    pub is_collecting_quotes: bool,
}

impl Data {
    pub fn new<S>(admins_file_name: S, quotes_file_name: S) -> Result<Self, std::io::Error>
    where
        S: Into<String>,
    {
        let mut result = Self {
            is_admin: HashSet::new(),
            admins: FileData::new(admins_file_name, true)?,
            quotes: FileData::new(quotes_file_name, true)?,
            is_collecting_quotes: false,
        };
        for id in result.admins.content.iter() {
            match id.trim().parse() {
                Err(_) => log::error!("Parsing error."),
                Ok(id) => {
                    result.is_admin.insert(id);
                }
            }
        }

        Ok(result)
    }
    pub fn is_admin(&self, userid: i64) -> bool {
        matches!(self.is_admin.get(&userid), Some(_))
    }
}

use teloxide::prelude::*;
use teloxide::types::User;
use teloxide::utils::command::BotCommand;

#[derive(BotCommand, PartialEq, Debug)]
#[command(rename = "lowercase", parse_with = "split")]
pub enum Command {
    Menu,
    Start,
    Startw,
    Stopw,
    Yuru(usize),
    Stop,
}

use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::keyboard::print_menu;
use crate::process::{answer, reply};

pub async fn execute_command(
    data: &Arc<Mutex<Data>>,
    message: Option<&UpdateWithCx<AutoSend<Bot>, Message>>,
    command: &Command,
    user: Option<&User>,
    is_terminal: bool,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut data = data.lock().await;
    let userid;
    let username;
    let is_admin = if is_terminal {
        username = String::from("Terminal");
        true
    } else if let Some(c) = user {
        userid = c.id;
        username = c.first_name.clone();
        data.is_admin(userid)
    } else {
        username = String::from("ERROR");
        false
    };
    match *command {
        Command::Startw => {
            if is_admin {
                data.is_collecting_quotes = true;
                if let Some(m) = message {
                    reply(m, "開始收集語錄^ ^").await;
                }
                log::info!("{} started the quotes collecting", username);
            } else {
                if let Some(m) = message {
                    reply(m, "缺少收集語錄權限^ ^").await;
                }
                log::info!("{} failed to start the quotes collecting", username);
            }
        }
        Command::Stopw => {
            if is_admin {
                data.is_collecting_quotes = false;
                if let Some(m) = message {
                    reply(m, "停止收集語錄^ ^").await;
                }
                log::info!("{} stopped the quotes collecting", username);
            } else {
                if let Some(m) = message {
                    reply(m, "缺少收集語錄權限^ ^").await;
                }
                log::info!("{} failed to stop the quotes collecting", username);
            }
        }
        Command::Yuru(index) => {
            let f = &data.quotes;
            let content = match &f.content.get(index - 1) {
                Some(s) => *s,
                None => {
                    if let Some(m) = message {
                        reply(m, "下標溢出或不存在。").await;
                    }
                    return Ok(());
                }
            };
            let content = match base64::decode(&content) {
                Ok(s) => String::from_utf8(s).unwrap_or_else(|_| content.clone()),
                Err(_) => content.clone(),
            };
            if let Some(m) = message {
                answer(m, content).await;
            }
        }
        Command::Menu => {
            print_menu(message, is_admin).await;
        }
        _ => {}
    }
    Ok(())
}
