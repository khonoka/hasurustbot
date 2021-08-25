use std::{error::Error, sync::Arc};

use teloxide::{
    types::{ChatKind, Me},
    utils::command::BotCommand,
};
use tokio::sync::Mutex;

use teloxide::prelude::*;
use tokio_stream::wrappers::UnboundedReceiverStream;

//use crate::keyboard::*;
use crate::{
    command::{execute_command, Command, Data},
    keyboard::display_qts,
};

pub async fn repl(
    bot: AutoSend<Bot>,
    data: &'static Arc<Mutex<Data>>,
) -> Result<(), std::io::Error> {
    Dispatcher::new(bot)
        .messages_handler(move |rx: DispatcherHandlerRx<AutoSend<Bot>, Message>| {
            UnboundedReceiverStream::new(rx).for_each_concurrent(None, move |message| async move {
                while let Err(e) = handle_message(&message, &data).await {
                    log::error!("{}", e);
                }
            })
        })
        .callback_queries_handler(
            move |rx: DispatcherHandlerRx<AutoSend<Bot>, CallbackQuery>| {
                UnboundedReceiverStream::new(rx).for_each_concurrent(
                    None,
                    move |query| async move {
                        let q = &query.update;
                        if let Some(callback) = &q.data {
                            if callback.starts_with("qtsto ") {
                                display_qts(&query, &data).await;
                            }
                            if callback.starts_with("blackhole") {
                                return;
                            }
                        }
                    },
                )
            },
        )
        .dispatch()
        .await;
    /*
        teloxide::repl(bot, move |message| async move {
            handle_message(message, &data).await;
            respond(())
        })
        .await;
    */

    Ok(())
}
async fn send_quotes(message: &UpdateWithCx<AutoSend<Bot>, Message>, data: &Arc<Mutex<Data>>) {
    let f = &data.lock().await.quotes;
    if f.content.is_empty() {
        return;
    }
    let content = &f.content[
            rand::random::<usize>()%f.content.len()
            //rand::thread_rng().gen_range(0..f.content.len()-1)
        ];
    let content = match base64::decode(&content) {
        Ok(s) => String::from_utf8(s).unwrap_or_else(|_| content.clone()),
        Err(_) => content.clone(),
    };
    reply(&message, content).await;
}
async fn check_send_quotes(bot_info: &Me, message: &UpdateWithCx<AutoSend<Bot>, Message>) -> bool {
    match message.update.reply_to_message() {
        Some(m) => {
            if let Some(u) = m.from() {
                if u.id != bot_info.user.id {
                    return false;
                }
            } else {
                return false;
            }
        }
        None => {
            if let ChatKind::Public(_) = message.update.chat.kind {
                return false;
            }
        }
    }
    true
}

pub async fn handle_message(
    message: &UpdateWithCx<AutoSend<Bot>, Message>,
    data: &Arc<Mutex<Data>>,
) -> Result<(), Box<dyn Error>> {
    let bot_info = match message.requester.get_me().await {
        Ok(me) => me,
        Err(e) => {
            log::error!("Unable to fetch the Bot's Information.{}", e);
            return Err(Box::new(e));
        }
    };
    //        log::info!("{:?}",message.update);
    print_message(&message);
    let text = message.update.text().unwrap_or("");
    let userid = match message.update.from() {
        Some(u) => u.id,
        None => 0,
    };
    if let Some(command_signal) = text.chars().next() {
        if command_signal == '/' {
            if let Ok(command) = Command::parse(text.trim(), "") {
                let user = message.update.from();
                if let Err(e) = execute_command(data, Some(&message), &command, user, false).await {
                    reply(&message, format!("執行時錯誤。{}", e)).await;
                }
                return Ok(());
            };
        }
    }
    if text.contains("貼貼") || text.contains("贴贴") {
        reply(&message, "貼貼^ ^").await;
    }
    if check_send_quotes(&bot_info, &message).await {
        send_quotes(&message, data).await;
    }
    let mut tmp_data = data.lock().await;
    if tmp_data.is_collecting_quotes && userid != 0 && tmp_data.is_admin(userid) && !text.is_empty()
    {
        let tmp_string = base64::encode(text);
        match tmp_data.quotes.save(tmp_string) {
            Ok(()) => {
                reply(&message, "已保存語錄^ ^").await;
                log::info!("Quote \"{}\" collected.", text);
            }
            Err(e) => {
                reply(&message, format!("保存語錄失敗- -:{}", e)).await;
                log::info!("Quote \"{}\" collecting failed:{}", text, e);
            }
        }
    }
    std::mem::drop(tmp_data);
    Ok(())
}

pub async fn reply<T>(message: &UpdateWithCx<AutoSend<Bot>, Message>, text: T)
where
    T: Into<String>,
{
    match message.reply_to(text).await {
        Ok(_) => (),
        Err(e) => {
            log::error!("Unable to Send Message.{}", e)
        }
    };
}
pub async fn answer<T>(message: &UpdateWithCx<AutoSend<Bot>, Message>, text: T)
where
    T: Into<String>,
{
    match message.answer(text).await {
        Ok(_) => (),
        Err(e) => {
            log::error!("Unable to Send Message.{}", e)
        }
    };
}

fn print_message(message: &UpdateWithCx<AutoSend<Bot>, Message>) {
    let owner_user = match message.update.from() {
        None => "Could not get the name of the user.",
        Some(user) => &user.first_name[..],
    };
    let st = message.update.text().unwrap_or("");
    println!("{}:{}", owner_user, st);
}
