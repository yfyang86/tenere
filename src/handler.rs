use crate::llm::{LLMAnswer, LLMRole};
use crate::{chat::Chat, prompt::Mode};

use crate::{
    app::{App, AppResult, FocusedBlock},
    event::Event,
    notification::{Notification, NotificationLevel},
};

use crate::llm::LLM;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use ratatui::text::Line;

use std::sync::Arc;
use tokio::sync::Mutex;

use tokio::sync::mpsc::UnboundedSender;

use std::fs::OpenOptions;
use std::io::Write;

use chrono::Local;

pub async fn handle_key_events(
    key_event: KeyEvent,
    app: &mut App<'_>,
    llm: Arc<Mutex<Box<dyn LLM + 'static>>>,
    sender: UnboundedSender<Event>,
) -> AppResult<()> {
    match key_event.code {
        // Quit the app
        KeyCode::Char('q') if app.prompt.mode != Mode::Insert => {
            app.running = false;
        }

        KeyCode::Char('c') if key_event.modifiers == KeyModifiers::CONTROL => {
            app.running = false;
        }

        //write the result to cache file
        KeyCode::Char('w') => {
            if let FocusedBlock::History = app.focused_block {
                if let Some(selected_index) = app.history.get_selected_index() {
                    let selected_history_prompts_and_chat = format!(
                        "{}\n{}",
                        app.history.text[selected_index].join("\n"),
                        app.history.preview.text[selected_index].to_string()
                    );
                    let file_path = app.config.file_path.history_file_path.to_string();
                    let now = Local::now()
                        .format("📅 [SESSION TIME] %Y-%m-%d %H:%M:%s")
                        .to_string();

                    let mut file = OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(file_path)?;

                    writeln!(
                        file,
                        "{}",
                        format!("{}\n{}", now, selected_history_prompts_and_chat)
                    )?;
                    let notif = Notification::new("写入历史".to_string(), NotificationLevel::Info);
                    sender.send(Event::Notification(notif)).unwrap();
                }
            } else {
                let current_result = app.chat.formatted_chat.to_string();
                let file_path = app.config.file_path.result_vault_path.to_string();
                let now = Local::now()
                    .format("📅 [SESSION TIME] %Y-%m-%d %H:%M:%s")
                    .to_string();

                let mut file = OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(file_path)?;
                writeln!(file, "{}", format!("{}\n{}", now, current_result))?;
                let notif = Notification::new("写入记录".to_string(), NotificationLevel::Info);
                sender.send(Event::Notification(notif)).unwrap();
            }
        }

        // Terminate the stream response
        KeyCode::Char('t') if key_event.modifiers == KeyModifiers::CONTROL => {
            app.terminate_response_signal
                .store(true, std::sync::atomic::Ordering::Relaxed);
        }

        // scroll down
        KeyCode::Char('j') | KeyCode::Down => match app.focused_block {
            FocusedBlock::History => {
                app.history.scroll_down();
            }

            FocusedBlock::Chat => {
                app.chat
                    .automatic_scroll
                    .store(false, std::sync::atomic::Ordering::Relaxed);
                app.chat.scroll = app.chat.scroll.saturating_add(1);
            }

            FocusedBlock::Preview => {
                app.history.preview.scroll = app.history.preview.scroll.saturating_add(1);
            }
            _ => (),
        },

        // scroll up
        KeyCode::Char('k') | KeyCode::Up => match app.focused_block {
            FocusedBlock::History => app.history.scroll_up(),

            FocusedBlock::Preview => {
                app.history.preview.scroll = app.history.preview.scroll.saturating_sub(1);
            }

            FocusedBlock::Chat => {
                app.chat
                    .automatic_scroll
                    .store(false, std::sync::atomic::Ordering::Relaxed);
                app.chat.scroll = app.chat.scroll.saturating_sub(1);
            }

            _ => (),
        },

        // `G`:  Mo to the bottom
        KeyCode::Char('G') => match app.focused_block {
            FocusedBlock::Chat => app.chat.move_to_bottom(),
            FocusedBlock::History => app.history.move_to_bottom(),
            _ => (),
        },

        // `gg`: Move to the top
        KeyCode::Char('g') => {
            if app.previous_key == KeyCode::Char('g') {
                match app.focused_block {
                    FocusedBlock::Chat => {
                        app.chat.move_to_top();
                    }
                    FocusedBlock::History => {
                        app.history.move_to_top();
                    }
                    _ => (),
                }
            }
        }

        // New chat
        KeyCode::Char(c)
            if c == app.config.key_bindings.new_chat
                && key_event.modifiers == KeyModifiers::CONTROL =>
        {
            app.prompt.clear();

            app.history
                .preview
                .text
                .push(app.chat.formatted_chat.clone());

            app.history.text.push(app.chat.plain_chat.clone());
            // after adding to history, save the chat in file
            app.history.save(app.history.text.len() - 1, sender.clone());

            app.chat = Chat::default();

            let llm = llm.clone();
            {
                let mut llm = llm.lock().await;
                llm.clear();
            }

            app.chat.scroll = 0;
        }

        // Switch the focus
        KeyCode::Tab => match app.focused_block {
            FocusedBlock::Chat => {
                app.focused_block = FocusedBlock::Prompt;

                app.chat
                    .automatic_scroll
                    .store(true, std::sync::atomic::Ordering::Relaxed);
            }
            FocusedBlock::Prompt => {
                app.chat.move_to_bottom();

                app.focused_block = FocusedBlock::Chat;
                app.prompt.mode = Mode::Normal;
            }
            FocusedBlock::History => {
                app.focused_block = FocusedBlock::Preview;
                app.history.preview.scroll = 0;
            }
            FocusedBlock::Preview => {
                app.focused_block = FocusedBlock::History;
                app.history.preview.scroll = 0;
            }
            _ => (),
        },

        // Show help
        KeyCode::Char(c)
            if c == app.config.key_bindings.show_help && app.prompt.mode != Mode::Insert =>
        {
            app.focused_block = FocusedBlock::Help;
            app.chat
                .automatic_scroll
                .store(true, std::sync::atomic::Ordering::Relaxed);
        }

        // Show history
        KeyCode::Char(c)
            if c == app.config.key_bindings.show_history
                && app.prompt.mode != Mode::Insert
                && key_event.modifiers == KeyModifiers::CONTROL =>
        {
            app.focused_block = FocusedBlock::History;
            app.chat
                .automatic_scroll
                .store(true, std::sync::atomic::Ordering::Relaxed);
        }

        // Discard help & history popups
        KeyCode::Esc => match app.focused_block {
            FocusedBlock::History | FocusedBlock::Preview | FocusedBlock::Help => {
                app.focused_block = FocusedBlock::Prompt
            }
            _ => {}
        },

        _ => {}
    }

    if let FocusedBlock::Prompt = app.focused_block {
        if let Mode::Normal = app.prompt.mode {
            if key_event.code == KeyCode::Enter {
                let user_input = app.prompt.editor.lines().join("\n");
                let user_input = user_input.trim();
                if user_input.is_empty() {
                    return Ok(());
                }

                app.prompt.clear();

                app.chat.plain_chat.push(format!("👤 : {}\n", user_input));

                if app.chat.formatted_chat.width() == 0 {
                    app.chat.formatted_chat = app
                        .formatter
                        .format(format!("👤: {}\n", user_input).as_str());
                } else {
                    app.chat.formatted_chat.extend(
                        app.formatter
                            .format(format!("👤: {}\n", user_input).as_str()),
                    );
                }

                let llm = llm.clone();
                {
                    let mut llm = llm.lock().await;
                    llm.append_chat_msg(user_input.into(), LLMRole::USER);
                }

                app.spinner.active = true;

                app.chat
                    .formatted_chat
                    .lines
                    .push(Line::raw("🤖: ".to_string()));

                let terminate_response_signal = app.terminate_response_signal.clone();

                let sender = sender.clone();

                let llm = llm.clone();

                tokio::spawn(async move {
                    let llm = llm.lock().await;
                    let res = llm.ask(sender.clone(), terminate_response_signal).await;

                    if let Err(e) = res {
                        sender
                            .send(Event::LLMEvent(LLMAnswer::StartAnswer))
                            .unwrap();
                        sender
                            .send(Event::LLMEvent(LLMAnswer::Answer(e.to_string())))
                            .unwrap();
                    }
                });
            }
        }

        app.prompt
            .handler(key_event, app.previous_key, app.clipboard.as_mut());
    }

    app.previous_key = key_event.code;

    Ok(())
}
