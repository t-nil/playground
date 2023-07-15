use std::{
    io::{self, Stdout},
    time::Duration,
};

use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use itertools::Itertools;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Chart, Dataset, GraphType},
    Terminal,
};

#[allow(unused)]
use ratatui::backend::{CrosstermBackend, TermionBackend, TermwizBackend};
use statistics::math::{Func, Plot};
#[warn(unused)]
use statistics::math::{self, Num, Point};

type BackendImpl = TermionBackend<Stdout>;

/// input poll takes duration, currently using fromMillis.
/// therefor convert FPS to tickrate
const FPS: u16 = 15;
const TICKRATE: u16 = 1000 / FPS;

const MIN: Num = -5.0;
const MAX: Num = 5.0;
const STEP: Num = 0.1;

static DATASETS: Mutex<LazyCell<Vec<(Plot<Point>, Dataset)>>> = Mutex::new(LazyCell::new(|| {
    let fns: Vec<(&str, Box<math::Func>)> = vec![
        ("x²", Box::new(|x| x * x)),
        ("x² moved on x axis", Box::new(|x| (x - 3.0) * (x - 3.0))),
        ("classical square root", Box::new(Num::sqrt)),
    ];
    // TODO bei ratatui beschweren, wie NaNs gerendert werden (see sqrt(-x))

    let datasets = fns
        .iter()
        .map(|(name, f)| {
            let plot = math::plot(f, MIN, MAX, STEP).collect_vec();
            (
                (*name, plot),
                Dataset::default()
                    .graph_type(GraphType::Line)
                    .marker(symbols::Marker::Braille)
                    .style(Style::default().fg(Color::Rgb(
                        rand::random::<u8>(),
                        rand::random::<u8>(),
                        rand::random::<u8>(),
                    )))
                    .name(*name),
            )
        })
        .collect_vec();
    datasets
}));

/// This is a bare minimum example. There are many approaches to running an application loop, so
/// this is not meant to be prescriptive. It is only meant to demonstrate the basic setup and
/// teardown of a terminal application.
///
/// A more robust application would probably want to handle errors and ensure that the terminal is
/// restored to a sane state before exiting. This example does not do that. It also does not handle
/// events or update the application state. It just draws a greeting and exits when the user
/// presses 'q'.
fn main() -> Result<()> {
    // calc datasets

    // spin up term
    let mut terminal = setup_terminal().context("setup failed")?;
    run(&mut terminal).context("app loop failed")?;
    restore_terminal(&mut terminal).context("restore terminal failed")?;
    Ok(())
}

/// Render the application. This is where you would draw the application UI. This example just
/// draws a greeting.
fn render_default_app(frame: &mut ratatui::Frame<BackendImpl>) {
    //let greeting = Paragraph::new("Hello World! (press 'q' to quit)");
    //frame.render_widget(greeting, frame.size());
    let size = frame.size();
    /*let _chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)].as_ref())
    .split(size);*/

    let (x_axis, y_axis) = (&[(MIN, 0.0), (MAX, 0.0)], &[(0.0, MIN), (0.0, MAX)]);

    let dataset = DATASETS.lock().unwrap();
    let chart = make_chart(
        dataset
            .iter()
            .map(|((_, plot), dataset)| dataset.clone().data(plot))
            .collect_vec(),
        x_axis,
        y_axis,
    );

    frame.render_widget(chart, size);
}

fn make_chart<'a>(
    mut charts: Vec<Dataset<'a>>,
    x_axis: &'a [Point; 2],
    y_axis: &'a [Point; 2],
) -> Chart<'a> {
    // IMPORTANT Dot seems to use `.` for drawing, Braille the UTF8 braille symbols (which can represent alot of dot configurations per symbol)
    // Block == fully filled symbol, Bar == "smaller" Block with emptiness at the top
    // at increasing dataset resolution, difference between GraphType::{Line, Scatter} disappears
    let mut datasets = vec![
        // also make x- and y-axis dynamic and oriented on the dataset

        // TODO BIG: make PR for ratatui for configuring the position of x and y axis
        // - already parameterized in ChartLayout::axis_x && axis_y
        // - calculated in Chart::layout() and applied in Chart::render()
        // would have to:
        // - correct the very procedural coordinate calculation with y -= 1, x -= 1 etc
        // - dynamically insert symbols::line::{BOTTOM_LEFT, TOP_RIGHT, CROSS}
        Dataset::default()
            .name("x")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .data(x_axis),
        Dataset::default()
            .name("y")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .data(y_axis),
    ];
    datasets.append(&mut charts);
    let chart = Chart::new(datasets)
        .block(Block::default().title("Chart"))
        .x_axis(
            Axis::default()
                .title(Span::styled("X Axis", Style::default().fg(Color::Red)))
                //.style(Style::default().fg(Color::White))
                .bounds([-1.0, 10.0])
                // TODO make dynamic and oriented on the dataset
                .labels(
                    ["-1.0", "4.5", "10.0"]
                        .iter()
                        .cloned()
                        .map(Span::from)
                        .collect(),
                ),
        )
        .y_axis(
            Axis::default()
                .title(Span::styled("Y Axis", Style::default().fg(Color::Red)))
                //.style(Style::default().fg(Color::White))
                .bounds([-1.0, 10.0])
                // TODO make dynamic and oriented on the dataset
                .labels(
                    ["-1", "4.5", "10"]
                        .iter()
                        .cloned()
                        .map(Span::from)
                        .collect(),
                ),
        );
    chart
}

/// Setup the terminal. This is where you would enable raw mode, enter the alternate screen, and
/// hide the cursor. This example does not handle errors. A more robust application would probably
/// want to handle errors and ensure that the terminal is restored to a sane state before exiting.
fn setup_terminal() -> Result<Terminal<BackendImpl>> {
    let mut stdout = io::stdout();
    enable_raw_mode().context("failed to enable raw mode")?;
    execute!(stdout, EnterAlternateScreen).context("unable to enter alternate screen")?;
    Terminal::new(BackendImpl::new(stdout)).context("creating terminal failed")
}

/// Restore the terminal. This is where you disable raw mode, leave the alternate screen, and show
/// the cursor.
fn restore_terminal(terminal: &mut Terminal<BackendImpl>) -> Result<()> {
    disable_raw_mode().context("failed to disable raw mode")?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .context("unable to switch to main screen")?;
    terminal.show_cursor().context("unable to show cursor")
}

/// Run the application loop. This is where you would handle events and update the application
/// state. This example exits when the user presses 'q'. Other styles of application loops are
/// possible, for example, you could have multiple application states and switch between them based
/// on events, or you could have a single application state and update it based on events.
fn run(terminal: &mut Terminal<BackendImpl>) -> Result<()> {
    loop {
        terminal.draw(crate::render_default_app)?;
        if should_quit()? {
            break;
        }
    }
    Ok(())
}

/// Check if the user has pressed 'q'. This is where you would handle events. This example just
/// checks if the user has pressed 'q' and returns true if they have. It does not handle any other
/// events. There is a 250ms timeout on the event poll so that the application can exit in a timely
/// manner, and to ensure that the terminal is rendered at least once every 250ms.
fn should_quit() -> Result<bool> {
    if event::poll(Duration::from_millis(TICKRATE.into())).context("event poll failed")? {
        if let Event::Key(key) = event::read().context("event read failed")? {
            return Ok(KeyCode::Char('q') == key.code);
        }
    }
    Ok(false)
}
