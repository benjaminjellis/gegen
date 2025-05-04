mod data_fetch;
mod logging;
mod state;
mod ui;

use color_eyre::Result;
use crossterm::event::{self};
use dashmap::DashMap;
use ratatui::DefaultTerminal;
use state::{PageRenderStates, State};
use std::{sync::Arc, thread::JoinHandle, time::Duration};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub(crate) const GEGEN_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();

    let file_appender = logging::create_file_appender();
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_writer(non_blocking),
        )
        .init();

    let data = Arc::new(DashMap::new());

    let (sender, recv) = crossbeam::channel::unbounded();

    let app_state = State::new(data.clone(), sender);

    let render_state = PageRenderStates::default();

    let data_join_handle = data_fetch::run_data_fetch(&data, app_state.today, recv);

    run(terminal, &data_join_handle, app_state, render_state)?;

    Ok(())
}

fn run(
    mut terminal: DefaultTerminal,
    data_join_handle: &JoinHandle<()>,
    mut app_state: State,
    mut page_states: PageRenderStates,
) -> Result<()> {
    loop {
        if app_state.should_quit || data_join_handle.is_finished() {
            break;
        }

        if event::poll(Duration::ZERO)? {
            let event = event::read()?;
            ui::process_event(event, &mut app_state, &mut page_states)
        }

        if app_state.should_draw() {
            tracing::info!("pre draw");
            app_state.on_tick(&mut page_states);
            terminal.draw(|frame| ui::draw_page(frame, &app_state, &mut page_states))?;

            tracing::info!("post draw");
        }
    }

    ratatui::restore();
    Ok(())
}
