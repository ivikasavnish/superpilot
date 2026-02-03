# Architecture

## Overview

Superpilot is a production-safe, low-latency Docker sidecar for black-box observability and optional traffic recording. It's designed to outperform kernel-hook based tools in production safety, latency, and footprint.

## Design Principles

1. **Observation must always be cheaper than recording**
2. **Recording must never affect production traffic**
3. **Recording must be physically impossible in prod build**

## Architecture Components

### 1. Core Components (Both Modes)

#### Config System (`config.rs`)
- YAML and environment variable support
- Type-safe configuration with serde
- Separate config structs for each component
- Default values for all settings

#### Metrics System (`metrics.rs`)
- Native Prometheus integration
- Low-cardinality metrics design
- Atomic counters and gauges (lock-free)
- Separate metrics for TCP, UDP, HTTP/1, HTTP/2, QUIC
- Safety metrics (rate limiting, sampling)

#### TCP Proxy (`tcp.rs`)
- Transparent L4 forwarding
- No protocol parsing (performance)
- Per-connection threads (no async overhead)
- Fixed buffers on hot path
- Non-blocking I/O
- Connection metrics

#### UDP Forwarder (`udp.rs`)
- Stateless forwarding
- Best-effort delivery
- Packet counting and drops tracking
- Size-limited packets

#### Safety Guard (`safety.rs`)
- Rate limiting (atomic counters)
- Sampling (deterministic)
- Auto-disable on resource pressure
- Lock-free implementation
- Per-second counter reset

#### Utility Functions (`util.rs`)
- Timestamp utilities
- Byte formatting
- Address parsing helpers

### 2. HTTP Components

#### HTTP/1 Observer (`http1.rs`)
- Minimal parsing for observability
- Method, path, status extraction
- Header size tracking
- Latency measurement
- **Prod mode**: Metadata only
- **Recorder mode**: Optional payload capture

#### HTTP/2 Observer (`http2.rs`)
- Connection preface detection
- Frame type identification
- Stream counting
- Reset tracking
- No full frame parsing (performance)

#### QUIC Observer (`quic.rs`)
- Packet format detection
- Long/short header identification
- Packet and byte counting
- Simplified RTT estimation

### 3. Recorder-Only Components

#### MITM Proxy (`mitm.rs`)
- **Only compiled with `recorder` feature**
- Certificate generation per host
- HTTP CONNECT handling
- CA certificate reuse
- Certificate caching

#### Recorder (`recorder.rs`)
- **Only compiled with `recorder` feature**
- Ring buffer implementation
- Async writer (never blocks proxy)
- Drop-on-backpressure semantics
- Atomic enable/disable
- Metadata vs. payload modes
- JSON Lines output format

## Feature Flag Architecture

### Cargo Features

```toml
[features]
default = ["prod"]
prod = []
recorder = []
```

### Compilation Strategy

- **Prod build**: Recording code is completely excluded at compile time
- **Recorder build**: All observability + recording capabilities
- Uses `#[cfg(feature = "...")]` attributes extensively
- Zero runtime overhead for disabled features

### What's Included

| Component | Prod Build | Recorder Build |
|-----------|-----------|----------------|
| TCP Proxy | ✓ | ✓ |
| UDP Forwarder | ✓ | ✓ |
| HTTP/1 Observer | ✓ (metadata) | ✓ (+ payload) |
| HTTP/2 Observer | ✓ | ✓ |
| QUIC Observer | ✓ | ✓ |
| MITM Proxy | ✗ | ✓ |
| Recorder | ✗ | ✓ |
| Metrics | ✓ (basic) | ✓ (+ recorder) |

## Performance Architecture

### Hot Path Optimizations

1. **No Heap Allocations**
   - Fixed-size buffers pre-allocated
   - Stack-based processing where possible
   - Buffer reuse across connections

2. **No Locks**
   - Atomic operations for counters
   - Per-thread resources
   - Lock-free data structures

3. **No Async Runtime**
   - Per-connection threads
   - Blocking I/O with timeouts
   - No tokio/async-std overhead

4. **Minimal Dependencies**
   - Only essential crates
   - No heavy frameworks
   - Static linking for deployment

### Memory Layout

```
┌─────────────────────────────────────┐
│  Main Thread                        │
│  - Metrics Server                   │
│  - TCP Listener                     │
└─────────────────────────────────────┘
           │
           ├── Connection Thread 1
           │   ├── Client Buffer (8KB)
           │   └── Target Buffer (8KB)
           │
           ├── Connection Thread 2
           │   ├── Client Buffer (8KB)
           │   └── Target Buffer (8KB)
           │
           └── ...
```

## Safety Architecture

### Multi-Layer Protection

1. **Rate Limiting**
   - Per-second connection/packet limits
   - Atomic counter-based
   - Automatic reset every second

2. **Sampling**
   - Configurable sampling rate (0.0-1.0)
   - Deterministic counter-based
   - No random number generation

3. **Size Limits**
   - Maximum header size
   - Maximum body size
   - Maximum packet size

4. **Auto-Disable (Recorder Only)**
   - Latency spike detection
   - Memory pressure detection
   - Disk fill detection
   - One-time disable per reason

### Resource Constraints

```yaml
Default Limits:
  - Memory: 256 MB
  - Latency: 100 ms
  - Rate: 10,000 req/s
  - Buffer: 8 KB (TCP)
  - Buffer: 64 KB (UDP)
```

## Metrics Architecture

### Prometheus Endpoint

- Native `/metrics` HTTP endpoint
- Text format (Prometheus standard)
- Low-cardinality labels
- Gauges, counters, histograms

### Metric Categories

1. **TCP Metrics**
   - Connection counts
   - Bytes in/out
   - Duration histograms
   - Error counts by type

2. **UDP Metrics**
   - Packet counts
   - Bytes in/out
   - Drop counts

3. **HTTP Metrics**
   - Request counts (by method, status)
   - Duration histograms
   - Size histograms

4. **HTTP/2 Metrics**
   - Stream counts
   - Reset counts
   - Byte counts

5. **QUIC Metrics**
   - Packet counts
   - Byte counts
   - RTT histograms

6. **Safety Metrics**
   - Rate-limited counts
   - Sampled-out counts
   - Memory usage

7. **Recorder Metrics** (recorder only)
   - Event counts
   - Bytes written
   - Drop counts
   - Auto-disable counts

## Deployment Architecture

### Docker

```
┌──────────────────────────────────────┐
│  Application Container               │
│  (Port 8081)                         │
└──────────────────────────────────────┘
                │
                ▼
┌──────────────────────────────────────┐
│  Superpilot Sidecar                  │
│  ├─ Listen: 8080                     │
│  ├─ Forward to: App:8081             │
│  └─ Metrics: 9090                    │
└──────────────────────────────────────┘
                │
                ▼
         External Clients
```

### Kubernetes

```yaml
Pod with Sidecar:
  - App Container (8081)
  - Superpilot Container (8080 → 8081)
  
Service exposes: 8080
Prometheus scrapes: 9090
```

## Code Organization

```
src/
├── main.rs              # Entry point, mode selection
├── config.rs            # Configuration system
├── metrics.rs           # Prometheus metrics
├── tcp.rs              # L4 TCP proxy
├── udp.rs              # L4 UDP forwarder
├── http1.rs            # HTTP/1.x observer
├── http2.rs            # HTTP/2 observer (observe-only)
├── quic.rs             # QUIC observer (observe-only)
├── mitm.rs             # TLS MITM (recorder only)
├── recorder.rs         # Recording system (recorder only)
├── safety.rs           # Safety guard
└── util.rs             # Utilities
```

## Security Considerations

1. **Default: No Payload Visibility**
   - Prod mode never sees request/response bodies
   - Only metadata (method, path, status, latency)

2. **MITM is Explicit Opt-In**
   - Requires recorder feature
   - Requires explicit configuration
   - Certificate management required

3. **No Credential Recording**
   - Authorization headers can be filtered
   - Cookie values can be redacted
   - Credential detection (future)

4. **Recorder Auto-Expires**
   - Configurable time limits
   - Auto-disable on issues
   - Manual disable supported

## Testing Strategy

### Unit Tests
- Per-module test coverage
- Configuration parsing
- Metric creation
- Safety guard logic
- Protocol detection

### Integration Tests
- End-to-end proxy functionality
- Metrics accuracy
- Safety limit enforcement
- Feature flag verification

### Performance Tests
- Latency overhead measurement
- Throughput testing
- Memory usage validation
- CPU usage profiling

## Future Enhancements

1. **Protocol Support**
   - gRPC detection
   - WebSocket tracking
   - Database protocol heuristics

2. **Advanced Observability**
   - Distributed tracing integration
   - Error classification
   - SLO tracking

3. **Enhanced Safety**
   - Circuit breaker
   - Adaptive rate limiting
   - Memory pressure detection

4. **Recorder Features**
   - Replay engine
   - Traffic shaping
   - Request/response matching
