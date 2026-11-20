//! A batch of netstat events laid out by a C compiler, read by the Rust SDK.
//!
//! `tests/c/netstat_events_fixture.c` fills one record of every event type with
//! the public header. This test compiles it with the system C compiler, runs
//! it, and reads the raw bytes it wrote through [`NetstatEvents`]. The records
//! are therefore laid out by C, not by the bindgen types the iterator reads
//! them through, so any disagreement in size, offset, or enum value fails here.

use std::path::{Path, PathBuf};
use std::process::Command;

use qcontrol::ffi::qcontrol_netstat_event_t;
use qcontrol::netstat::{
    Direction, Family, Instance, InstanceEnd, NetEndpoint, NetstatEventKind, NetstatEvents,
    TransportTotals, UnknownCause,
};
use qcontrol::NetstatProtocol;

/// The pid the fixture puts in every record.
const FIXTURE_PID: u32 = 4242;
/// The instance number of the fixture's socket events.
const FIXTURE_INSTANCE: Instance = Instance(7);
/// The time of the fixture's first record; each later record is one
/// nanosecond after the one before.
const FIXTURE_FIRST_NS: u64 = 1000;
/// Wall-clock nanoseconds the fixture's watermark carries.
const FIXTURE_WALL_NS: u64 = 1_700_000_000_000_000_000;
/// The type number the fixture sends that no SDK version defines.
const FIXTURE_FUTURE_TYPE: u16 = 60000;

/// The system C compiler: `$CC` when set, otherwise `cc`.
fn c_compiler() -> String {
    std::env::var("CC").unwrap_or_else(|_| "cc".to_string())
}

/// Compile and run the fixture, returning the records it wrote.
fn build_batch() -> Vec<qcontrol_netstat_event_t> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let scratch = Path::new(env!("CARGO_TARGET_TMPDIR")).join("netstat_events_c");
    std::fs::create_dir_all(&scratch).expect("create scratch dir");
    let exe = scratch.join(if cfg!(windows) {
        "fixture.exe"
    } else {
        "fixture"
    });
    let out = scratch.join("batch.bin");

    let compiled = Command::new(c_compiler())
        .arg("-I")
        .arg(manifest.join("include"))
        .arg("-o")
        .arg(&exe)
        .arg(manifest.join("tests/c/netstat_events_fixture.c"))
        .status()
        .expect("the system C compiler must be runnable to build the C fixture");
    assert!(compiled.success(), "C fixture failed to compile");

    let ran = Command::new(&exe)
        .arg(&out)
        .status()
        .expect("run C fixture");
    assert!(ran.success(), "C fixture failed to run");

    let bytes = std::fs::read(&out).expect("read batch");
    let size = std::mem::size_of::<qcontrol_netstat_event_t>();
    assert_eq!(bytes.len() % size, 0, "batch is whole records");
    bytes
        .chunks_exact(size)
        .map(|chunk| unsafe { std::ptr::read_unaligned(chunk.as_ptr().cast()) })
        .collect()
}

/// An IPv4 or IPv6 endpoint with only the first address byte set.
fn endpoint(family: Family, port: u16, first: u8) -> NetEndpoint {
    let mut addr = [0; 16];
    addr[0] = first;
    NetEndpoint { family, port, addr }
}

#[test]
fn a_batch_built_in_c_reads_back_field_for_field() {
    let records = build_batch();
    let batch = unsafe { NetstatEvents::from_raw(records.as_ptr(), records.len()) };
    let events: Vec<_> = batch.iter().collect();

    let instance = FIXTURE_INSTANCE;
    let expected = [
        NetstatEventKind::Opened {
            instance,
            protocol: NetstatProtocol::Tcp,
            family: Family::Ipv4,
            preexisting: true,
        },
        NetstatEventKind::Connecting {
            instance,
            peer: Some(endpoint(Family::Ipv4, 443, 10)),
        },
        NetstatEventKind::Connected {
            instance,
            peer: Some(endpoint(Family::Ipv6, 443, 32)),
        },
        NetstatEventKind::ConnectFailed {
            instance,
            peer: None,
        },
        NetstatEventKind::ConnectAbandoned { instance },
        NetstatEventKind::LocalEndpoint {
            instance,
            endpoint: endpoint(Family::Ipv4, 55000, 127),
        },
        NetstatEventKind::PeerEndpoint {
            instance,
            endpoint: endpoint(Family::Ipv6, 8443, 253),
        },
        NetstatEventKind::Transferred {
            instance,
            direction: Direction::Send,
            bytes: 1500,
            peer: None,
            late: false,
            name_not_open: false,
        },
        NetstatEventKind::TransferFailed {
            instance,
            direction: Direction::Recv,
            peer: Some(endpoint(Family::Ipv4, 53, 8)),
        },
        NetstatEventKind::NamesClosed { instance },
        NetstatEventKind::Ended {
            instance,
            reason: InstanceEnd::Replaced,
        },
        NetstatEventKind::ProcessArmed,
        NetstatEventKind::ProcessEnded,
        NetstatEventKind::Watermark {
            wall_ns: FIXTURE_WALL_NS,
        },
        NetstatEventKind::Unknown {
            from_ns: 10,
            to_ns: 20,
            cause: UnknownCause::RecordsLost,
        },
        NetstatEventKind::Losses {
            transport: TransportTotals {
                submitted: 100,
                ring_full: 1,
                too_large: 2,
                payload_pool_exhausted: 3,
                invalid: 4,
            },
        },
        NetstatEventKind::Stopped,
        NetstatEventKind::Transferred {
            instance,
            direction: Direction::Recv,
            bytes: 9,
            peer: Some(NetEndpoint::UNSPECIFIED),
            late: true,
            name_not_open: true,
        },
        NetstatEventKind::Unrecognized(FIXTURE_FUTURE_TYPE),
    ];

    assert_eq!(events.len(), expected.len());
    for (index, (event, kind)) in events.iter().zip(expected).enumerate() {
        assert_eq!(event.pid, FIXTURE_PID, "record {index}");
        assert_eq!(
            event.at_ns,
            FIXTURE_FIRST_NS + index as u64,
            "record {index}"
        );
        assert_eq!(event.kind, kind, "record {index}");
    }
}
