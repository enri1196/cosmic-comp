// SPDX-License-Identifier: GPL-3.0-only

use crate::state::Common;
use libsystemd::daemon::{booted, notify, NotifyState};
use tracing::{error, warn};
use zbus::{blocking::Connection, Result};

#[zbus::proxy(
    interface = "org.freedesktop.systemd1.Manager",
    default_service = "org.freedesktop.systemd1",
    default_path = "/org/freedesktop/systemd1",
    gen_blocking = true
)]
trait Manager {
    fn set_environment(&self, assignments: &[&str]) -> zbus::Result<()>;
}

pub fn ready(common: &Common) {
    if !booted() {
        return;
    }

    if let Err(err) = set_systemd_environment(common) {
        warn!(?err, "Failed to import WAYLAND_DISPLAY/DISPLAY into systemd via D-Bus");
    }

    if let Err(err) = notify(false, &[NotifyState::Ready]) {
        error!(?err, "Failed to notify systemd that service is ready");
    }
}

fn set_systemd_environment(common: &Common) -> Result<()> {
    let connection = Connection::session()?;
    let proxy = ManagerProxyBlocking::new(&connection)?;

    let mut env_vars = vec![format!("WAYLAND_DISPLAY={}", &common.socket.to_str().unwrap_or_default())];

    if let Some(s) = common.xwayland_state.as_ref() {
        env_vars.push(format!("DISPLAY=:{}", s.display));
    }

    let env_vars_ref: Vec<&str> = env_vars.iter().map(|s| s.as_str()).collect();

    proxy.set_environment(&env_vars_ref)?;

    Ok(())
}
