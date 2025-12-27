use crate::consts::LOGS_FILENAME;
use log::LevelFilter;
use std::env;

pub fn setup_logger() -> Result<(), fern::InitError> {
    let log_file = env::current_exe()?.parent().unwrap().join(LOGS_FILENAME);

    #[cfg(debug_assertions)]
    let debug = true;
    #[cfg(not(debug_assertions))]
    let debug = false;

    let dispatch = fern::Dispatch::new();

    let dispatch = if debug {
        dispatch.level(LevelFilter::Trace)
    }
    else {
        dispatch.level(LevelFilter::Info)
    };

    dispatch
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} {} {}: {}",
                jiff::Timestamp::now().strftime("%I:%M:%S %d.%m.%Y"),
                record.level(),
                record.target(),
                message
            ))
        })
        .chain(std::io::stdout())
        .chain(fern::log_file(log_file)?)
        .apply()?;

    Ok(())
}
