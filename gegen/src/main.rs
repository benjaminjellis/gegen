mod data_fetch;
mod state;
mod ui;

use color_eyre::Result;
use crossterm::event::{self};
use dashmap::DashMap;
use ratatui::DefaultTerminal;
use state::State;
use std::{sync::Arc, thread::JoinHandle, time::Duration};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub(crate) const GEGEN_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();

    let file_appender = tracing_appender::rolling::minutely("./.logs", "gegen.log");

    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // let env_filter = EnvFilter::builder()
    //     .with_default_directive(LevelFilter::INFO.into())
    //     .parse_lossy("gegen=info");

    tracing_subscriber::registry()
        // .with(env_filter)
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_writer(non_blocking),
        )
        .init();

    let data = Arc::new(DashMap::new());

    let (sender, recv) = crossbeam::channel::unbounded();

    let app_state = State::new(data.clone(), sender);

    let data_join_handle = data_fetch::run_data_fetch(&data, app_state.today, recv);

    // pre fetch data
    app_state.fetch_data_for_date(app_state.today);
    tracing::info!("pre fetch");

    run(terminal, &data_join_handle, app_state)?;

    Ok(())
}

fn run(
    mut terminal: DefaultTerminal,
    data_join_handle: &JoinHandle<()>,
    mut app_state: State,
) -> Result<()> {
    loop {
        if app_state.should_quit {
            break;
        }

        if data_join_handle.is_finished() {
            tracing::error!("gegen quit due to the data fetching thread completeting");
            break;
        }

        if event::poll(Duration::ZERO)? {
            let event = event::read()?;
            ui::process_event(event, &mut app_state)
        }

        if app_state.should_draw() {
            tracing::info!("pre draw");
            app_state.on_tick();
            terminal.draw(|frame| ui::draw_page(frame, &mut app_state))?;

            tracing::info!("post draw");
        }
    }

    ratatui::restore();
    Ok(())
}
