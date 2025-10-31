use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
};
use textwrap::{Options, wrap};

pub struct Tui {
    pub input: String,
    pub history: Vec<String>,
    pub scroll: u16,
}

impl Tui {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            history: Vec::new(),
            scroll: 0,
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let size = frame.size();
        let options = Options::new((size.width.saturating_sub(2)) as usize)
            .break_words(true);

        let wrapped_input = wrap(&self.input, &options).join("\n");
        let input_height = (wrapped_input.lines().count() as u16).max(3);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [Constraint::Min(1), Constraint::Length(input_height)].as_ref(),
            )
            .split(size);

        let history_text = self.history.join("\n");
        let wrapped_history = history_text
            .lines()
            .map(|line| wrap(line, &options).join("\n"))
            .collect::<Vec<_>>()
            .join("\n");

        let history_height = wrapped_history.lines().count() as u16;
        let available_height = chunks[0].height.saturating_sub(2); // subtract borders
        self.scroll =
            self.scroll.min(history_height.saturating_sub(available_height));

        let history = Paragraph::new(Text::from(wrapped_history))
            .block(Block::default().borders(Borders::ALL).title("History"))
            .scroll((self.scroll, 0));
        frame.render_widget(history, chunks[0]);

        let input = Paragraph::new(Text::from(wrapped_input))
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title("Input"));
        frame.render_widget(input, chunks[1]);
    }

    pub fn add_to_history(&mut self, msg: String) {
        self.history.push(msg);
    }
}
