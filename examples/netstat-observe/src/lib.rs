//! Example plugin that registers the netstat event callback.
//!
//! The callback counts the events of each type in a batch and prints one line
//! per batch. Batch boundaries carry no meaning, so a plugin that wants a
//! measurement aggregates across batches. The batch is valid only for this
//! call.

use std::collections::BTreeMap;

use qcontrol::prelude::*;

/// The name printed for an event's type.
fn type_name(kind: &NetstatEventKind) -> &'static str {
    match kind {
        NetstatEventKind::Opened { .. } => "opened",
        NetstatEventKind::Connecting { .. } => "connecting",
        NetstatEventKind::Connected { .. } => "connected",
        NetstatEventKind::ConnectFailed { .. } => "connect_failed",
        NetstatEventKind::ConnectAbandoned { .. } => "connect_abandoned",
        NetstatEventKind::LocalEndpoint { .. } => "local_endpoint",
        NetstatEventKind::PeerEndpoint { .. } => "peer_endpoint",
        NetstatEventKind::Transferred { .. } => "transferred",
        NetstatEventKind::TransferFailed { .. } => "transfer_failed",
        NetstatEventKind::NamesClosed { .. } => "names_closed",
        NetstatEventKind::Ended { .. } => "ended",
        NetstatEventKind::ProcessArmed => "process_armed",
        NetstatEventKind::ProcessEnded => "process_ended",
        NetstatEventKind::Watermark { .. } => "watermark",
        NetstatEventKind::Unknown { .. } => "unknown",
        NetstatEventKind::Losses { .. } => "losses",
        NetstatEventKind::Stopped => "stopped",
        NetstatEventKind::Unrecognized(_) => "unrecognized",
    }
}

fn on_netstat(events: &NetstatEvents<'_>) {
    let mut counts: BTreeMap<&'static str, usize> = BTreeMap::new();
    for event in events {
        *counts.entry(type_name(&event.kind)).or_default() += 1;
    }
    let mut line = format!("netstat events={}", events.len());
    for (name, count) in &counts {
        line.push_str(&format!(" {name}={count}"));
    }
    eprintln!("{line}");
}

export_plugin!(PluginBuilder::new("netstat-observe").on_netstat(on_netstat));
