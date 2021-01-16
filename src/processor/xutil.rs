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

pub struct XUtil {}

impl XUtil {
    pub fn create_connections(
    ) -> Result<(impl Connection + Send + Sync, impl Connection + Send + Sync), Box<dyn Error>>
    {
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

    pub fn create_record_context<C>(
        ctx: &Context,
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

        match ctx.timeout_sec {
            None => {}
            Some(timeout) => {
                std::thread::spawn(move || {
                    debug!("start timer({} sec)", timeout);
                    std::thread::sleep(std::time::Duration::from_secs(timeout));
                    debug!("diable record context");
                    ctrl_conn.record_disable_context(record_context).unwrap();
                    ctrl_conn.sync().unwrap();
                });
            }
        }
        Ok(())
    }
}
