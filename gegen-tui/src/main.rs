mod data_fetch;
mod state;
mod ui;

use color_eyre::Result;
use crossterm::event::{self};
use ratatui::DefaultTerminal;
use state::State;
use std::{
    sync::{Arc, RwLock},
    thread::JoinHandle,
    time::Duration,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> Result<()> {
    let data = Arc::new(RwLock::new(None));

    let data_join_handle = data_fetch::run_data_fetch(&data);

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

    let ui_state = State::new(data);

    run(terminal, &data_join_handle, ui_state)?;

    // if data_join_handle.is_finished() {}
    Ok(())
}

fn run(
    mut terminal: DefaultTerminal,
    _: &JoinHandle<Result<(), String>>,
    mut app_state: State,
) -> Result<()> {
    loop {
        if app_state.should_quit {
            break;
        }

        if event::poll(Duration::ZERO)? {
            let event = event::read()?;
            ui::process_event(event, &mut app_state)
        }

        if app_state.should_draw() {
            tracing::info!("pre draw");
            app_state.on_tick();
            terminal
                .draw(|frame| ui::draw_page(frame, &mut app_state))
                .unwrap();

            tracing::info!("post draw");
        }
    }

    ratatui::restore();
    Ok(())
}
