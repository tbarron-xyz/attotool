use crate::attotool::loop_tools_until_finish;
use crate::event::{Event, Events};
use crate::tool_handler::{LoopError, PromptType};
use crate::tui::Tui;
use crossterm::{
    event::EnableMouseCapture,
    execute,
    terminal::{
        EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use tokio::time::{Duration, interval};

pub struct App {
    pub tui: Tui,
    pub should_quit: bool,
    pub model: String,
    pub retries: u32,
    pub max_tokens: u32,
    pub max_tool_calls: u32,
    pub base_url: String,
    pub verbose: bool,
    pub tool_call_details: bool,
    pub disable_agents_md: bool,
    pub yolo: bool,
    pub r#continue: bool,
    pub plan: bool,
    pub ui_mode: bool,
    pub processing: bool,
    pub pending_prompt: Option<PromptType>,
    pub pending_input: Option<String>,
    pub pending_approval: Option<bool>,
    pub tool_calls: Vec<(String, String)>,
}

impl App {
    pub fn new(
        model: String,
        retries: u32,
        max_tokens: u32,
        max_tool_calls: u32,
        base_url: String,
        verbose: bool,
        tool_call_details: bool,
        disable_agents_md: bool,
        yolo: bool,
        r#continue: bool,
        plan: bool,
        ui_mode: bool,
    ) -> Self {
        Self {
            tui: Tui::new(),
            should_quit: false,
            model,
            retries,
            max_tokens,
            max_tool_calls,
            base_url,
            verbose,
            tool_call_details,
            disable_agents_md,
            yolo,
            r#continue,
            plan,
            ui_mode,
            processing: false,
            pending_prompt: None,
            pending_input: None,
            pending_approval: None,
            tool_calls: Vec::new(),
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut stdout = io::stdout();
        enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen)?;
        execute!(stdout, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let mut events = Events::new();
        let mut interval = interval(Duration::from_millis(250));
        let (output_tx, mut output_rx) = tokio::sync::mpsc::channel(100);
        let (result_tx, mut result_rx) = tokio::sync::mpsc::channel::<
            Result<
                Option<crate::tool_handler::PromptType>,
                Box<dyn std::error::Error + Send + Sync>,
            >,
        >(1);

        terminal.draw(|f| self.tui.draw(f))?; // initial draw

        loop {
            tokio::select! {
                event = events.next() => {
                    match event? {
                        Event::Input(key) => {
                            if self.processing && self.pending_prompt.is_none() {
                                // Ignore inputs while processing initial request
                                continue;
                            }
                            match key {
                              crossterm::event::KeyCode::Char('y') => {
                                   if let Some(PromptType::Approval(_)) = &self.pending_prompt {
                                       self.pending_approval = Some(true);
                                       self.tui.input.push('y');
                                   } else {
                                       self.tui.input.push('y');
                                   }
                               }
                              crossterm::event::KeyCode::Char('Y') => {
                                   if let Some(PromptType::Approval(_)) = &self.pending_prompt {
                                       self.pending_approval = Some(true);
                                       self.tui.input.push('Y');
                                   } else {
                                       self.tui.input.push('Y');
                                   }
                               }
                              crossterm::event::KeyCode::Char('n') => {
                                   if let Some(PromptType::Approval(_)) = &self.pending_prompt {
                                       self.pending_approval = Some(false);
                                       self.tui.input.push('n');
                                   } else {
                                       self.tui.input.push('n');
                                   }
                               }
                              crossterm::event::KeyCode::Char('N') => {
                                   if let Some(PromptType::Approval(_)) = &self.pending_prompt {
                                       self.pending_approval = Some(false);
                                       self.tui.input.push('N');
                                   } else {
                                       self.tui.input.push('N');
                                   }
                               }
                            crossterm::event::KeyCode::Char('\x04') => {
                                self.should_quit = true;
                            }
                             crossterm::event::KeyCode::Char(c) => {
                                 if self.pending_prompt.is_none() || matches!(self.pending_prompt, Some(PromptType::Clarification(_))) {
                                     self.tui.input.push(c);
                                 }
                             }
                             crossterm::event::KeyCode::Backspace => {
                                 if self.pending_prompt.is_none() || matches!(self.pending_prompt, Some(PromptType::Clarification(_))) {
                                     self.tui.input.pop();
                                 }
                             }
                             crossterm::event::KeyCode::Up => {
                                 if self.pending_prompt.is_none() {
                                     if let Some(last_user_msg) = self.tui.history.iter().rev().find(|s| s.starts_with("User: ")) {
                                         let msg = last_user_msg.strip_prefix("User: ").unwrap_or(last_user_msg);
                                         self.tui.input = msg.to_string();
                                     }
                                 }
                             }
                             crossterm::event::KeyCode::Enter => {
                                 if let Some(prompt_type) = &self.pending_prompt {
                                     match prompt_type {
                                          PromptType::Approval(_) => {
                                              if let Some(approval) = self.pending_approval.take() {
                                               if let Some(input) = self.pending_input.take() {
                                                   self.processing = true;
                                                   self.tui.add_to_history("Processing approval...".to_string());
                                                   let tx = result_tx.clone();
                                                   let output_tx_clone = output_tx.clone();
                                                   let model = self.model.clone();
                                                   let retries = self.retries;
                                                   let max_tokens = self.max_tokens;
                                                   let max_tool_calls = self.max_tool_calls;
                                                   let base_url = self.base_url.clone();
                                                   let verbose = self.verbose;
                                                   let tool_call_details = self.tool_call_details;
                                                   let disable_agents_md = self.disable_agents_md;
                                                   let yolo = self.yolo;
                                                   let r#continue = self.r#continue;
                                                   let plan = self.plan;
                                                   let ui_mode = self.ui_mode;
                                                   tokio::spawn(async move {
                                                       let mut output = crate::output::TuiOutput { sender: output_tx_clone };
                                                       let mut tool_calls = Vec::new();
                                                       let result = loop_tools_until_finish(
                                                           input,
                                                           &model,
                                                           retries,
                                                           max_tokens,
                                                           max_tool_calls,
                                                           &base_url,
                                                           verbose,
                                                           tool_call_details,
                                                           disable_agents_md,
                                                           yolo,
                                                           r#continue,
                                                           plan,
                                                           ui_mode,
                                                           Some(approval),
                                                           &mut output,
                                                           &mut tool_calls,
                                                       ).await;
                                                       let prompt_result = match result {
                                                           Ok(_) => Ok(None),
                                                           Err(LoopError::Prompt(p)) => Ok(Some(p)),
                                                           Err(LoopError::Other(e)) => Err(e),
                                                       };
                                                       let _ = tx.send(prompt_result).await;
                                                   });
                                               }
                                              }
                                              self.tui.input.clear();
                                          }
                                          PromptType::Clarification(_) => {
                                              let answer = self.tui.input.clone();
                                              self.tui.add_to_history(format!("User: {}", answer.clone()));
                                              self.tui.input.clear();
                                              terminal.draw(|f| self.tui.draw(f))?; // immediate redraw
                                              self.processing = true;
                                              self.tui.add_to_history("Processing clarification...".to_string());
                                              let tx = result_tx.clone();
                                              let output_tx_clone = output_tx.clone();
                                              let model = self.model.clone();
                                              let retries = self.retries;
                                              let max_tokens = self.max_tokens;
                                              let max_tool_calls = self.max_tool_calls;
                                              let base_url = self.base_url.clone();
                                              let verbose = self.verbose;
                                              let tool_call_details = self.tool_call_details;
                                              let disable_agents_md = self.disable_agents_md;
                                              let yolo = self.yolo;
                                              let r#continue = self.r#continue;
                                              let plan = self.plan;
                                              let ui_mode = self.ui_mode;
                                              tokio::spawn(async move {
                                                  let mut output = crate::output::TuiOutput { sender: output_tx_clone };
                                                  let mut tool_calls = Vec::new();
                                                  let result = loop_tools_until_finish(
                                                      answer,
                                                      &model,
                                                      retries,
                                                      max_tokens,
                                                      max_tool_calls,
                                                      &base_url,
                                                      verbose,
                                                      tool_call_details,
                                                      disable_agents_md,
                                                      yolo,
                                                      r#continue,
                                                      plan,
                                                      ui_mode,
                                                      None,
                                                      &mut output,
                                                      &mut tool_calls,
                                                  ).await;
                                                  let prompt_result = match result {
                                                      Ok(_) => Ok(None),
                                                      Err(LoopError::Prompt(p)) => Ok(Some(p)),
                                                      Err(LoopError::Other(e)) => Err(e),
                                                  };
                                                  let _ = tx.send(prompt_result).await;
                                              });
                                         }
                                     }
                                 } else {
                                     if !self.processing {
                                         let input = self.tui.input.clone();
                                         self.tui.add_to_history(format!("User: {}", input.clone()));
                                         self.tui.input.clear();
                                         terminal.draw(|f| self.tui.draw(f))?; // immediate redraw
                                         self.processing = true;
                                         self.tui.add_to_history("Processing request...".to_string());
                                         let tx = result_tx.clone();
                                         let output_tx_clone = output_tx.clone();
                                         let model = self.model.clone();
                                         let retries = self.retries;
                                         let max_tokens = self.max_tokens;
                                         let max_tool_calls = self.max_tool_calls;
                                         let base_url = self.base_url.clone();
                                         let verbose = self.verbose;
                                         let tool_call_details = self.tool_call_details;
                                         let disable_agents_md = self.disable_agents_md;
                                         let yolo = self.yolo;
                                         let r#continue = self.r#continue;
                                         let plan = self.plan;
                                         let ui_mode = self.ui_mode;
                                         tokio::spawn(async move {
                                             let mut output = crate::output::TuiOutput { sender: output_tx_clone };
                                             let mut tool_calls = Vec::new();
                                             let result = loop_tools_until_finish(
                                                 input,
                                                 &model,
                                                 retries,
                                                 max_tokens,
                                                 max_tool_calls,
                                                 &base_url,
                                                 verbose,
                                                 tool_call_details,
                                                 disable_agents_md,
                                                 yolo,
                                                 r#continue,
                                                 plan,
                                                 ui_mode,
                                                 None,
                                                 &mut output,
                                                 &mut tool_calls,
                                             ).await;
                                             let prompt_result = match result {
                                                 Ok(_) => Ok(None),
                                                 Err(LoopError::Prompt(p)) => Ok(Some(p)),
                                                 Err(LoopError::Other(e)) => Err(e),
                                             };
                                             let _ = tx.send(prompt_result).await;
                                         });
                                     }
                                 }
                             }
                            crossterm::event::KeyCode::Esc => {
                                self.should_quit = true;
                            }
                             _ => {}
                            }
                            terminal.draw(|f| self.tui.draw(f))?; // redraw after input
                        }
                        Event::ScrollUp => {
                            self.tui.scroll = self.tui.scroll.saturating_sub(1);
                            terminal.draw(|f| self.tui.draw(f))?;
                        }
                        Event::ScrollDown => {
                            self.tui.scroll = self.tui.scroll.saturating_add(1);
                            terminal.draw(|f| self.tui.draw(f))?;
                        }
                        Event::Tick => {}
                    }
                }
                msg = output_rx.recv() => {
                    if let Some(msg) = msg {
                        self.tui.add_to_history(msg);
                        terminal.draw(|f| self.tui.draw(f))?;
                    }
                }
                result = result_rx.recv() => {
                    if let Some(result) = result {
                        self.processing = false;
                        match result {
                            Ok(Some(prompt_type)) => {
                                match &prompt_type {
                                    PromptType::Approval(msg) => self.tui.add_to_history(format!("Approval: {}", msg)),
                                    PromptType::Clarification(msg) => self.tui.add_to_history(format!("Clarification: {}", msg)),
                                }
                                if matches!(prompt_type, PromptType::Clarification(_)) {
                                    self.tui.input.clear();
                                }
                                self.pending_prompt = Some(prompt_type);
                            }
                            Ok(None) => {}
                            Err(e) => {
                                self.tui.add_to_history(format!("Error: {}", e));
                            }
                        }
                        terminal.draw(|f| self.tui.draw(f))?;
                    }
                }
                _ = interval.tick() => {
                    terminal.draw(|f| self.tui.draw(f))?; // periodic redraw
                }
            }

            if self.should_quit {
                break;
            }
        }

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        Ok(())
    }
}
