use super::context::Context;
use std::error::Error;
use std::sync::Arc;
use x11rb::connection::Connection;

use x11rb::connection::RequestConnection;

use super::error::XcapeError;
use x11rb::protocol::record::{self, ConnectionExt as _};
use x11rb::protocol::xproto;
use x11rb::protocol::xtest::{self};

use x11rb::wrapper::ConnectionExt;
use x11rb::x11_utils::TryParse;

fn create_connections(
) -> Result<(impl Connection + Send + Sync, impl Connection + Send + Sync), Box<dyn Error>> {
    let (ctrl_conn, _) = x11rb::connect(None)?;
    let (data_conn, _) = x11rb::connect(None)?;

    if ctrl_conn
        .extension_information(record::X11_EXTENSION_NAME)?
        .is_none()
    {
        return Err(XcapeError::XConnectionInitError(
            "xrecord is not supported".to_string(),
        ))?;
    }

    if ctrl_conn
        .extension_information(xtest::X11_EXTENSION_NAME)?
        .is_none()
    {
        return Err(XcapeError::XConnectionInitError(
            "xtest is not supported".to_string(),
        ))?;
    }

    match ctrl_conn.extension_information(xtest::X11_EXTENSION_NAME)? {
        Some(inf) => debug!("xtest: {:?}", inf),
        None => {
            eprintln!("")
        }
    }
    Ok((ctrl_conn, data_conn))
}

struct RecordConf {
    range: record::Range,
}

impl RecordConf {
    fn new() -> Self {
        let empty = record::Range8 { first: 0, last: 0 };
        let empty_ext = record::ExtRange {
            major: empty,
            minor: record::Range16 { first: 0, last: 0 },
        };
        let range = record::Range {
            core_requests: empty,
            core_replies: empty,
            ext_requests: empty_ext,
            ext_replies: empty_ext,
            delivered_events: empty,
            device_events: record::Range8 {
                first: xproto::KEY_PRESS_EVENT,
                last: xproto::MOTION_NOTIFY_EVENT,
            },
            errors: empty,
            client_started: false,
            client_died: false,
        };

        RecordConf { range: range }
    }
}

struct XConnections<'a, C, D> {
    ctrl: &'a C,
    data: &'a D,
}

impl<'a, C, D> XConnections<'a, C, D>
where
    C: Connection + Send + Sync + 'static,
    D: Connection + Send + Sync + 'static,
{
    fn new(ctrl_conn: &'a C, data_conn: &'a D) -> Self {
        XConnections {
            ctrl: ctrl_conn,
            data: data_conn,
        }
    }
}

fn create_record_context<C>(
    ctrl_conn: Arc<C>,
    record_context: record::Context, //u32
) -> Result<(), Box<dyn Error>>
where
    C: Connection + Send + Sync + 'static,
{
    let record_conf = RecordConf::new();
    ctrl_conn
        .record_create_context(
            record_context,
            0,
            &[record::CS::AllClients.into()],
            &[record_conf.range],
        )?
        .check()?;

    // Apply a timeout if we are requested to do so.
    match std::env::var("X11RB_EXAMPLE_TIMEOUT")
        .ok()
        .and_then(|str| str.parse().ok())
    {
        None => {}
        Some(timeout) => {
            let cloned_connection = Arc::clone(&ctrl_conn);
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_secs(timeout));
                cloned_connection
                    .record_disable_context(record_context)
                    .unwrap();
                cloned_connection.sync().unwrap();
            });
        }
    }
    Ok(())
}

pub fn run(ctx: Arc<Context>) -> Result<(), Box<dyn Error>> {
    let connections = create_connections()?;
    let ctrl_conn = Arc::new(connections.0);
    let data_conn = Arc::new(connections.1);

    let rc = ctrl_conn.generate_id()?;
    create_record_context(ctrl_conn, rc)?;

    println!("main logic here {:?}", ctx.is_debug_mode());
    Ok(())
}
