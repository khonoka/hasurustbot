use std::sync::Arc;

use teloxide::prelude::*;
mod command;
mod file_data;
mod keyboard;
mod process;
mod terminal;

use process::*;
use terminal::*;
use tokio::{select, sync::Mutex};

use crate::command::Data;

#[macro_use]
extern crate lazy_static;
lazy_static! {
    static ref DATA: Arc<Mutex<Data>> = Arc::new(Mutex::new(
        Data::new("savedata/admins.lst", "savedata/quotes.lst").unwrap()
    ));
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    teloxide::enable_logging!();
    log::info!("Starting hasuchanbot...");

    let bot = Bot::from_env().auto_send();
    print_bot_info(&bot).await;
    let handle_message = repl(bot, &DATA);
    let terminal = handle_terminal(&DATA);
    select! {
        _ = handle_message => (),
        _ = terminal => (),
    };
    //try_join!(handle_message, terminal)?;
    Ok(())
}
