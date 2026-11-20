/*
 * Test fixture: builds one batch of netstat events the way a C host lays them
 * out and writes the raw records to the file named by argv[1].
 *
 * The Rust test compiles this with the system C compiler against the SDK's
 * synced header, runs it, and reads the bytes back with the Rust iterator, so
 * the layout the C compiler produces is what the Rust SDK has to agree with.
 * One record of every event type is written, in type-number order, followed by
 * extra records that exercise the optional fields.
 */

#include <stdio.h>
#include <string.h>

#include "qcontrol/netstat.h"

/* The pid every record is about. */
#define FIXTURE_PID 4242u
/* Instance number of the socket events. */
#define FIXTURE_INSTANCE 7u
/* Time of the first record; each record is one nanosecond after the last. */
#define FIXTURE_FIRST_NS 1000u
/* Most records the fixture writes. */
#define FIXTURE_MAX_RECORDS 32

static qcontrol_netstat_event_t records[FIXTURE_MAX_RECORDS];
static size_t count = 0;

/* Start the next record with the common fields filled and the rest zero. */
static qcontrol_netstat_event_t* next_record(uint16_t type, uint16_t flags) {
    qcontrol_netstat_event_t* ev = &records[count];
    memset(ev, 0, sizeof(*ev));
    ev->type = type;
    ev->flags = flags;
    ev->pid = FIXTURE_PID;
    ev->at_ns = FIXTURE_FIRST_NS + count;
    ev->instance = FIXTURE_INSTANCE;
    count++;
    return ev;
}

/* An endpoint whose first address byte is `first` and whose port is `port`. */
static qcontrol_netstat_endpoint_t make_endpoint(uint8_t family, uint16_t port, uint8_t first) {
    qcontrol_netstat_endpoint_t ep;
    memset(&ep, 0, sizeof(ep));
    ep.family = family;
    ep.port = port;
    ep.addr[0] = first;
    return ep;
}

int main(int argc, char** argv) {
    if (argc != 2) {
        return 2;
    }
    qcontrol_netstat_event_t* ev;

    ev = next_record(QCONTROL_NETSTAT_EVENT_OPENED, QCONTROL_NETSTAT_EVENT_PREEXISTING);
    ev->body.opened.protocol = QCONTROL_NETSTAT_PROTO_TCP;
    ev->body.opened.family = QCONTROL_NETSTAT_ADDR_IPV4;

    ev = next_record(QCONTROL_NETSTAT_EVENT_CONNECTING, QCONTROL_NETSTAT_EVENT_HAS_PEER);
    ev->body.connect.peer = make_endpoint(QCONTROL_NETSTAT_ADDR_IPV4, 443, 10);

    ev = next_record(QCONTROL_NETSTAT_EVENT_CONNECTED, QCONTROL_NETSTAT_EVENT_HAS_PEER);
    ev->body.connect.peer = make_endpoint(QCONTROL_NETSTAT_ADDR_IPV6, 443, 32);

    ev = next_record(QCONTROL_NETSTAT_EVENT_CONNECT_FAILED, 0);

    next_record(QCONTROL_NETSTAT_EVENT_CONNECT_ABANDONED, 0);

    ev = next_record(QCONTROL_NETSTAT_EVENT_LOCAL_ENDPOINT, 0);
    ev->body.endpoint.endpoint = make_endpoint(QCONTROL_NETSTAT_ADDR_IPV4, 55000, 127);

    ev = next_record(QCONTROL_NETSTAT_EVENT_PEER_ENDPOINT, 0);
    ev->body.endpoint.endpoint = make_endpoint(QCONTROL_NETSTAT_ADDR_IPV6, 8443, 253);

    ev = next_record(QCONTROL_NETSTAT_EVENT_TRANSFERRED, 0);
    ev->body.transfer.bytes = 1500;
    ev->body.transfer.direction = QCONTROL_NETSTAT_DIR_SEND;

    ev = next_record(QCONTROL_NETSTAT_EVENT_TRANSFER_FAILED, QCONTROL_NETSTAT_EVENT_HAS_PEER);
    ev->body.transfer.direction = QCONTROL_NETSTAT_DIR_RECV;
    ev->body.transfer.peer = make_endpoint(QCONTROL_NETSTAT_ADDR_IPV4, 53, 8);

    next_record(QCONTROL_NETSTAT_EVENT_NAMES_CLOSED, 0);

    ev = next_record(QCONTROL_NETSTAT_EVENT_ENDED, 0);
    ev->body.ended.reason = QCONTROL_NETSTAT_INSTANCE_REPLACED;

    next_record(QCONTROL_NETSTAT_EVENT_PROCESS_ARMED, 0);
    next_record(QCONTROL_NETSTAT_EVENT_PROCESS_ENDED, 0);

    ev = next_record(QCONTROL_NETSTAT_EVENT_WATERMARK, 0);
    ev->body.watermark.wall_ns = 1700000000000000000ull;

    ev = next_record(QCONTROL_NETSTAT_EVENT_UNKNOWN, 0);
    ev->body.unknown.from_ns = 10;
    ev->body.unknown.to_ns = 20;
    ev->body.unknown.cause = QCONTROL_NETSTAT_UNKNOWN_RECORDS_LOST;

    ev = next_record(QCONTROL_NETSTAT_EVENT_LOSSES, 0);
    ev->body.losses.submitted = 100;
    ev->body.losses.ring_full = 1;
    ev->body.losses.too_large = 2;
    ev->body.losses.payload_pool_exhausted = 3;
    ev->body.losses.invalid = 4;

    next_record(QCONTROL_NETSTAT_EVENT_STOPPED, 0);

    /* A late transfer through a closed name, with a datagram peer. */
    ev = next_record(QCONTROL_NETSTAT_EVENT_TRANSFERRED,
                     QCONTROL_NETSTAT_EVENT_HAS_PEER | QCONTROL_NETSTAT_EVENT_LATE |
                         QCONTROL_NETSTAT_EVENT_NAME_NOT_OPEN);
    ev->body.transfer.bytes = 9;
    ev->body.transfer.direction = QCONTROL_NETSTAT_DIR_RECV;
    ev->body.transfer.peer = make_endpoint(QCONTROL_NETSTAT_ADDR_UNSPEC, 0, 0);

    /* A type number a newer host might send. */
    next_record(60000, 0);

    FILE* out = fopen(argv[1], "wb");
    if (out == NULL) {
        return 3;
    }
    size_t written = fwrite(records, sizeof(records[0]), count, out);
    int closed = fclose(out);
    return (written == count && closed == 0) ? 0 : 4;
}
