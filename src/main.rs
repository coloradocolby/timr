use clap::Parser;
use std::{
    error::Error,
    io,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc, Arc,
    },
    thread,
    time::Duration,
};
use termion::{event::Key, input::TermRead, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::{Backend, TermionBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::Style,
    text::Span,
    widgets::Paragraph,
    Frame, Terminal,
};

#[derive(Parser, Debug, Clone)]
#[clap(version, about, long_about= None)]
pub struct Args {
    // the duration of the timer in seconds
    #[clap(short = 'd', long, default_value_t = 10)]
    duration: usize,
}

#[derive(Debug, Clone)]
struct App {
    duration: Arc<AtomicUsize>,
}

impl App {
    fn new(args: Args) -> Self {
        Self {
            duration: Arc::new(AtomicUsize::from(args.duration)),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let stdout = io::stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(args);

    let events = key_events();
    loop {
        terminal.draw(|f| draw(f, &mut app).unwrap()).unwrap();
        match events.recv()? {
            key => match key {
                Key::Char('\n') => {
                    thread::spawn(move || loop {
                        if app.duration.load(Ordering::SeqCst) == 0 {
                            break;
                        }
                        terminal.draw(|f| draw(f, &mut app).unwrap()).unwrap();
                        thread::sleep(Duration::from_secs(1));
                        app.duration.fetch_sub(1, Ordering::Relaxed); //atomic version of -=1 for a counter
                    });
                }
                Key::Char('q') => {
                    return Ok(());
                }
                _ => {}
            },
        }
    }
}

fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) -> Result<(), ()> {
    let h = &f.size().height;
    let height_of_timer = 1.;
    let mar = ((*h as f64 - height_of_timer) / 2.) as u16;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(mar as u16),
                Constraint::Length(height_of_timer as u16),
                Constraint::Length(mar as u16),
            ]
            .as_ref(),
        )
        .split(f.size());

    f.render_widget(
        Paragraph::new(Span::styled(
            String::from(format!("{:?}", app.duration)),
            Style::default(),
        ))
        .alignment(Alignment::Center),
        chunks[1],
    );

    Ok(())
}

fn key_events() -> mpsc::Receiver<Key> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let stdin = io::stdin();
        for key in stdin.keys().flatten() {
            if let Err(err) = tx.send(key) {
                eprintln!("{}", err);
                return;
            }
        }
    });

    rx
}
