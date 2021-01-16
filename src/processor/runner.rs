use super::context::Context;
use super::state::State;
use super::xutil::XUtil;
use std::error::Error;
use std::sync::Arc;
use x11rb::connection::Connection;

use x11rb::protocol::record::{self, ConnectionExt as _};
use x11rb::protocol::xproto;
use x11rb::protocol::xtest::{self};

use x11rb::x11_utils::TryParse;


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
            let _old_state = state.press_key(key);
            Ok(remaining)
        }
        xproto::KEY_RELEASE_EVENT => {
            let (event, remaining) = xproto::KeyReleaseEvent::try_parse(data)?;
            debug!("KeyRelease: {}", event.detail);
            let key = event.detail;
            let _old_value = state.release_key(key);
            Ok(remaining)
        }
        xproto::BUTTON_PRESS_EVENT => {
            let (event, remaining) = xproto::ButtonPressEvent::try_parse(data)?;
            debug!("ButtonPress: {}", event.detail);
            state.press_mouse();
            Ok(remaining)
        }
        xproto::BUTTON_RELEASE_EVENT => {
            let (event, remaining) = xproto::ButtonReleaseEvent::try_parse(data)?;
            debug!("ButtonRelease: {}", event.detail);
            state.release_mouse();
            Ok(remaining)
        }
        _ => Ok(&data[32..]),
    }
}

pub fn run(ctx: &Context) -> Result<(), Box<dyn Error>> {
    let connections = XUtil::create_connections()?;
    let ctrl_conn = Arc::new(connections.0);
    let data_conn = Arc::new(connections.1);

    let record_context = ctrl_conn.generate_id()?;
    XUtil::create_record_context(ctx, Arc::clone(&ctrl_conn), record_context)?;
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
