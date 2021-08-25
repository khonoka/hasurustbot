use std::{str::from_utf8, sync::Arc, usize};

use teloxide::{
    payloads::{EditMessageTextSetters, SendMessageSetters},
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardButtonKind, InlineKeyboardMarkup},
};
use tokio::sync::Mutex;

use crate::command::Data;

pub async fn print_menu(message: Option<&UpdateWithCx<AutoSend<Bot>, Message>>, _is_admin: bool) {
    let button = InlineKeyboardButton::new(
        "語錄列表",
        InlineKeyboardButtonKind::CallbackData(String::from("qtsto 1")),
    );
    let keyboard = InlineKeyboardMarkup::default().append_row(vec![button]);
    if let Some(m) = message {
        if m.answer("菜單").reply_markup(keyboard).await.is_ok() {
            //log::info!("Menu Display Ok");
        }
    }
}
pub async fn display_qts(
    query: &UpdateWithCx<AutoSend<Bot>, CallbackQuery>,
    data: &Arc<Mutex<Data>>,
) {
    let quotes = &data.lock().await.quotes;
    let mut command;
    if let Some(s) = &query.update.data {
        command = s.split(' ');
    } else {
        command = "".split(' ');
    }
    let index: usize;
    command.next();
    match command.next() {
        Some(s) => match s.parse() {
            Ok(tmp) => {
                index = tmp;
            }
            Err(_) => {
                log::info!("Display: Invalid number:{}", s);
                return;
            }
        },
        None => {
            log::info!("Display: No index.");
            return;
        }
    }
    if index < 1 || index > (quotes.content.len() + 9) / 10 {
        return;
    }
    let button_names = vec![String::from("<<"), index.to_string(), String::from(">>")];
    let callback_data = vec![
        format!("qtsto {}", index - 1),
        String::from("blackhole"),
        format!("qtsto {}", index + 1),
    ];
    let mut keyboard = InlineKeyboardMarkup::default();
    let mut buttons = vec![];
    for i in 0..3 {
        buttons.push(InlineKeyboardButton::new(
            &button_names[i],
            InlineKeyboardButtonKind::CallbackData(callback_data[i].clone()),
        ));
    }
    keyboard = keyboard.append_row(buttons);

    let mut content = String::from("語錄\n");
    let mut i = 0;
    while i + (index - 1) * 10 < quotes.content.len() && i < 10 {
        let tmp_string = &quotes.content[i + (index - 1) * 10];
        content.push_str(&(i + 1 + (index - 1) * 10).to_string());
        content.push('.');
        match base64::decode(tmp_string) {
            Ok(res) => {
                content.push_str(&from_utf8(&res).unwrap_or(tmp_string));
            }
            Err(_) => {
                content.push_str(tmp_string);
            }
        }
        content.push('\n');
        i += 1;
    }
    if let Some(message) = &query.update.message {
        //let t=query.requester.edit_message_text(message.chat_id()
        //, message.id, content).reply_markup(keyboard).await;
        if query
            .requester
            .edit_message_text(message.chat_id(), message.id, content)
            .reply_markup(keyboard)
            .await
            .is_ok()
        {
            //log::info!("Quotes Display Ok");
        }
    }
}
