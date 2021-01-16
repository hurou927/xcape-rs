use super::context::Context;
use core::cell::Cell;
use core::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
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

fn create_record_context<C>(
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

#[derive(Debug)]
struct KeyState {
    fake_keys: Vec<u8>,
    is_pressed: bool,
    is_used: bool,
}

impl KeyState {
    fn new(fake_keys: Vec<u8>) -> Self {
        KeyState {
            fake_keys: fake_keys,
            is_pressed: false,
            is_used: false,
        }
    }
}

#[derive(Debug)]
struct State {
    is_mouse_pressed: Cell<bool>,
    key_map: RefCell<HashMap<u8, KeyState>>,
}

impl State {
    fn new(ctx: &Context) -> Self {
        let key_map = ctx
            .key_map
            .iter()
            .map(|(k, v)| (*k, KeyState::new(v.clone())))
            .collect();
        State {
            is_mouse_pressed: Cell::new(false),
            key_map: RefCell::new(key_map),
        }
    }

    fn update_is_mouse_pressed(&self, is_pressed: bool) {
        self.is_mouse_pressed.set(is_pressed)
    }
    fn get_is_mouse_pressed(&self) -> bool {
        self.is_mouse_pressed.get()
    }

    // return old value
    fn update_key_pressed(&self, key: u8, is_pressed: bool) -> Option<bool> {
        let old = match self.key_map.borrow_mut().entry(key) {
            Entry::Occupied(o) => {
                let new = o.into_mut();
                let old = new.is_pressed;
                new.is_pressed = is_pressed;
                Some(old)
            }
            Entry::Vacant(_) => None,
        };
        debug!("udpate key. key:{}, new:{}, old:{:?}", key, is_pressed, old);
        old
    }

    fn release_key(&self, key: u8) {
        match self.key_map.borrow_mut().entry(key) {
            Entry::Occupied(o) => {
                let new = o.into_mut();
                debug!("Update State: key:{}, is_pressed:{}, is_used:{}", key, new.is_pressed, new.is_used);
                new.is_pressed = false;
                new.is_used = false;
            }
            Entry::Vacant(_) => {} 
        };
    }

    fn update_key_used(&self, is_used: bool) {
        for (_, val) in self.key_map.borrow_mut().iter_mut() {
            if val.is_pressed { 
                val.is_used = true;
            }
        }
    }
}

type Data = [u8];
fn intercept<'a, C>(
    _ctx: &Context,
    state: &State,
    data: &'a Data,
    ctrl_conn: Arc<C>,
) -> Result<&'a Data, Box<dyn Error>>
where
    C: Connection + Send + Sync + 'static,
{
    match data[0] {
        xproto::KEY_PRESS_EVENT => {
            let (event, remaining) = xproto::KeyPressEvent::try_parse(data)?;
            debug!("KeyPress: {}", event.detail);
            let key = event.detail;
            let _old_state = state.update_key_pressed(key, true);
            Ok(remaining)
        }
        xproto::KEY_RELEASE_EVENT => {
            let (event, remaining) = xproto::KeyReleaseEvent::try_parse(data)?;
            debug!("KeyRelease: {}", event.detail);
            let key = event.detail;
            state.release_key(key);
            Ok(remaining)
        }
        xproto::BUTTON_PRESS_EVENT => {
            let (event, remaining) = xproto::ButtonPressEvent::try_parse(data)?;
            debug!("ButtonPress: {}", event.detail);
            state.update_is_mouse_pressed(true);
            Ok(remaining)
        }
        xproto::BUTTON_RELEASE_EVENT => {
            let (event, remaining) = xproto::ButtonReleaseEvent::try_parse(data)?;
            debug!("ButtonRelease: {}", event.detail);
            state.update_is_mouse_pressed(false);
            Ok(remaining)
        }
        _ => Ok(&data[32..]),
    }
}

pub fn run(ctx: &Context) -> Result<(), Box<dyn Error>> {
    let connections = create_connections()?;
    let ctrl_conn = Arc::new(connections.0);
    let data_conn = Arc::new(connections.1);

    let record_context = ctrl_conn.generate_id()?;
    create_record_context(ctx, Arc::clone(&ctrl_conn), record_context)?;
    const START_OF_DATA: u8 = 4;
    const RECORD_FROM_SERVER: u8 = 0;
    let state = State::new(&ctx);
    for reply in data_conn.record_enable_context(record_context)? {
        let reply = reply?;
        if reply.client_swapped {
            warn!("Byte swapped clients are unsupported");
        } else if reply.category == RECORD_FROM_SERVER {
            let mut remaining = &reply.data[..];
            while !remaining.is_empty() {
                remaining = intercept(&ctx, &state, &reply.data, Arc::clone(&ctrl_conn))?;
            }
        } else if reply.category == START_OF_DATA {
            debug!("Start Of Date");
        } else {
            warn!("Got a reply with an unsupported category: {:?}", reply);
        }
    }

    println!("main logic here {:?}", ctx.is_debug_mode());
    Ok(())
}
