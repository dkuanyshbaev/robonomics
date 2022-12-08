use futures::future::BoxFuture;
use futures::prelude::*;
use futures_timer::Delay;
use instant::Instant;
use libp2p::{
    core::{
        connection::ConnectionId,
        upgrade::{NegotiationError, ReadyUpgrade},
        UpgradeError,
    },
    swarm::{
        ConnectionHandler, ConnectionHandlerEvent, ConnectionHandlerUpgrErr, KeepAlive,
        NegotiatedSubstream, NetworkBehaviour, NetworkBehaviourAction, PollParameters,
        SubstreamProtocol,
    },
    PeerId,
};
use rand::{distributions, prelude::*};
use std::{
    collections::VecDeque,
    error::Error,
    fmt, io,
    num::NonZeroU32,
    task::{Context, Poll},
    time::Duration,
};
use void::Void;

// pub const PROTOCOL_NAME: &[u8] = b"/robonomics/ros/1.0.0";
pub const PROTOCOL_NAME: &[u8] = b"/ipfs/ping/1.0.0";

#[derive(Default, Debug, Copy, Clone)]
pub struct Ping;

const PING_SIZE: usize = 32;

/// Sends a ping and waits for the pong.
pub async fn send_ping<S>(mut stream: S) -> io::Result<(S, Duration)>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let payload: [u8; PING_SIZE] = thread_rng().sample(distributions::Standard);
    stream.write_all(&payload).await?;
    stream.flush().await?;
    let started = Instant::now();
    let mut recv_payload = [0u8; PING_SIZE];
    stream.read_exact(&mut recv_payload).await?;
    if recv_payload == payload {
        Ok((stream, started.elapsed()))
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Ping payload mismatch",
        ))
    }
}

/// Waits for a ping and sends a pong.
pub async fn recv_ping<S>(mut stream: S) -> io::Result<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let mut payload = [0u8; PING_SIZE];
    stream.read_exact(&mut payload).await?;
    stream.write_all(&payload).await?;
    stream.flush().await?;
    Ok(stream)
}

/// The result of an inbound or outbound ping.
pub type Result = std::result::Result<Success, Failure>;

/// A [`NetworkBehaviour`] that responds to inbound pings and
/// periodically sends outbound pings on every established connection.
///
/// See the crate root documentation for more information.
pub struct Behaviour {
    /// Configuration for outbound pings.
    config: Config,
    /// Queue of events to yield to the swarm.
    events: VecDeque<RosEvent>,
}

/// Event generated by the `Ping` network behaviour.
#[derive(Debug)]
pub struct RosEvent {
    /// The peer ID of the remote.
    pub peer: PeerId,
    /// The result of an inbound or outbound ping.
    pub result: Result,
}

impl Behaviour {
    /// Creates a new `Ping` network behaviour with the given configuration.
    pub fn new(config: Config) -> Self {
        Self {
            config,
            events: VecDeque::new(),
        }
    }
}

impl Default for Behaviour {
    fn default() -> Self {
        Self::new(Config::new())
    }
}

impl NetworkBehaviour for Behaviour {
    type ConnectionHandler = Handler;
    type OutEvent = RosEvent;

    fn new_handler(&mut self) -> Self::ConnectionHandler {
        Handler::new(self.config.clone())
    }

    fn inject_event(&mut self, peer: PeerId, _: ConnectionId, result: Result) {
        self.events.push_front(RosEvent { peer, result })
    }

    fn poll(
        &mut self,
        _: &mut Context<'_>,
        _: &mut impl PollParameters,
    ) -> Poll<NetworkBehaviourAction<Self::OutEvent, Self::ConnectionHandler>> {
        if let Some(e) = self.events.pop_back() {
            let RosEvent { result, peer } = &e;

            match result {
                Ok(Success::Ping { .. }) => log::debug!("Ping sent to {:?}", peer),
                Ok(Success::Pong) => log::debug!("Ping received from {:?}", peer),
                _ => {}
            }

            Poll::Ready(NetworkBehaviourAction::GenerateEvent(e))
        } else {
            Poll::Pending
        }
    }
}

// use futures::future::BoxFuture;
// use futures::prelude::*;
// use futures_timer::Delay;
// use libp2p::core::upgrade::ReadyUpgrade;
// use libp2p::core::{upgrade::NegotiationError, UpgradeError};
// use libp2p::swarm::{
//     ConnectionHandler, ConnectionHandlerEvent, ConnectionHandlerUpgrErr, KeepAlive,
//     NegotiatedSubstream, SubstreamProtocol,
// };
// use std::{error::Error, fmt, num::NonZeroU32};
// use void::Void;

/// The configuration for outbound pings.
#[derive(Debug, Clone)]
pub struct Config {
    /// The timeout of an outbound ping.
    timeout: Duration,
    /// The duration between the last successful outbound or inbound ping
    /// and the next outbound ping.
    interval: Duration,
    /// The maximum number of failed outbound pings before the associated
    /// connection is deemed unhealthy, indicating to the `Swarm` that it
    /// should be closed.
    max_failures: NonZeroU32,
    /// Whether the connection should generally be kept alive unless
    /// `max_failures` occur.
    keep_alive: bool,
}

impl Config {
    /// Creates a new [`Config`] with the following default settings:
    ///
    ///   * [`Config::with_interval`] 15s
    ///   * [`Config::with_timeout`] 20s
    ///   * [`Config::with_max_failures`] 1
    ///   * [`Config::with_keep_alive`] false
    ///
    /// These settings have the following effect:
    ///
    ///   * A ping is sent every 15 seconds on a healthy connection.
    ///   * Every ping sent must yield a response within 20 seconds in order to
    ///     be successful.
    ///   * A single ping failure is sufficient for the connection to be subject
    ///     to being closed.
    ///   * The connection may be closed at any time as far as the ping protocol
    ///     is concerned, i.e. the ping protocol itself does not keep the
    ///     connection alive.
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_secs(20),
            interval: Duration::from_secs(15),
            max_failures: NonZeroU32::new(1).expect("1 != 0"),
            keep_alive: false,
        }
    }

    /// Sets the ping timeout.
    pub fn with_timeout(mut self, d: Duration) -> Self {
        self.timeout = d;
        self
    }

    /// Sets the ping interval.
    pub fn with_interval(mut self, d: Duration) -> Self {
        self.interval = d;
        self
    }

    /// Sets the maximum number of consecutive ping failures upon which the remote
    /// peer is considered unreachable and the connection closed.
    pub fn with_max_failures(mut self, n: NonZeroU32) -> Self {
        self.max_failures = n;
        self
    }

    /// Sets whether the ping protocol itself should keep the connection alive,
    /// apart from the maximum allowed failures.
    ///
    /// By default, the ping protocol itself allows the connection to be closed
    /// at any time, i.e. in the absence of ping failures the connection lifetime
    /// is determined by other protocol handlers.
    ///
    /// If the maximum number of allowed ping failures is reached, the
    /// connection is always terminated as a result of [`ConnectionHandler::poll`]
    /// returning an error, regardless of the keep-alive setting.
    #[deprecated(
        since = "0.40.0",
        note = "Use `libp2p::swarm::behaviour::KeepAlive` if you need to keep connections alive unconditionally."
    )]
    pub fn with_keep_alive(mut self, b: bool) -> Self {
        self.keep_alive = b;
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// The successful result of processing an inbound or outbound ping.
#[derive(Debug)]
pub enum Success {
    /// Received a ping and sent back a pong.
    Pong,
    /// Sent a ping and received back a pong.
    ///
    /// Includes the round-trip time.
    Ping { rtt: Duration },
}

/// An outbound ping failure.
#[derive(Debug)]
pub enum Failure {
    /// The ping timed out, i.e. no response was received within the
    /// configured ping timeout.
    Timeout,
    /// The peer does not support the ping protocol.
    Unsupported,
    /// The ping failed for reasons other than a timeout.
    Other {
        error: Box<dyn std::error::Error + Send + 'static>,
    },
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Failure::Timeout => f.write_str("Ping timeout"),
            Failure::Other { error } => write!(f, "Ping error: {}", error),
            Failure::Unsupported => write!(f, "Ping protocol not supported"),
        }
    }
}

impl Error for Failure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Failure::Timeout => None,
            Failure::Other { error } => Some(&**error),
            Failure::Unsupported => None,
        }
    }
}

/// Protocol handler that handles pinging the remote at a regular period
/// and answering ping queries.
///
/// If the remote doesn't respond, produces an error that closes the connection.
pub struct Handler {
    /// Configuration options.
    config: Config,
    /// The timer used for the delay to the next ping as well as
    /// the ping timeout.
    timer: Delay,
    /// Outbound ping failures that are pending to be processed by `poll()`.
    pending_errors: VecDeque<Failure>,
    /// The number of consecutive ping failures that occurred.
    ///
    /// Each successful ping resets this counter to 0.
    failures: u32,
    /// The outbound ping state.
    outbound: Option<OutboundState>,
    /// The inbound pong handler, i.e. if there is an inbound
    /// substream, this is always a future that waits for the
    /// next inbound ping to be answered.
    inbound: Option<PongFuture>,
    /// Tracks the state of our handler.
    state: State,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    /// We are inactive because the other peer doesn't support ping.
    Inactive {
        /// Whether or not we've reported the missing support yet.
        ///
        /// This is used to avoid repeated events being emitted for a specific connection.
        reported: bool,
    },
    /// We are actively pinging the other peer.
    Active,
}

impl Handler {
    /// Builds a new [`Handler`] with the given configuration.
    pub fn new(config: Config) -> Self {
        Handler {
            config,
            timer: Delay::new(Duration::new(0, 0)),
            pending_errors: VecDeque::with_capacity(2),
            failures: 0,
            outbound: None,
            inbound: None,
            state: State::Active,
        }
    }
}

impl ConnectionHandler for Handler {
    type InEvent = Void;
    type OutEvent = Result;
    type Error = Failure;
    type InboundProtocol = ReadyUpgrade<&'static [u8]>;
    type OutboundProtocol = ReadyUpgrade<&'static [u8]>;
    type OutboundOpenInfo = ();
    type InboundOpenInfo = ();

    fn listen_protocol(&self) -> SubstreamProtocol<ReadyUpgrade<&'static [u8]>, ()> {
        SubstreamProtocol::new(ReadyUpgrade::new(PROTOCOL_NAME), ())
    }

    fn inject_fully_negotiated_inbound(&mut self, stream: NegotiatedSubstream, (): ()) {
        self.inbound = Some(recv_ping(stream).boxed());
    }

    fn inject_fully_negotiated_outbound(&mut self, stream: NegotiatedSubstream, (): ()) {
        self.timer.reset(self.config.timeout);
        self.outbound = Some(OutboundState::Ping(send_ping(stream).boxed()));
    }

    fn inject_event(&mut self, _: Void) {}

    fn inject_dial_upgrade_error(&mut self, _info: (), error: ConnectionHandlerUpgrErr<Void>) {
        self.outbound = None; // Request a new substream on the next `poll`.

        let error = match error {
            ConnectionHandlerUpgrErr::Upgrade(UpgradeError::Select(NegotiationError::Failed)) => {
                debug_assert_eq!(self.state, State::Active);

                self.state = State::Inactive { reported: false };
                return;
            }
            // Note: This timeout only covers protocol negotiation.
            ConnectionHandlerUpgrErr::Timeout => Failure::Timeout,
            e => Failure::Other { error: Box::new(e) },
        };

        self.pending_errors.push_front(error);
    }

    fn connection_keep_alive(&self) -> KeepAlive {
        if self.config.keep_alive {
            KeepAlive::Yes
        } else {
            KeepAlive::No
        }
    }

    fn poll(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<ConnectionHandlerEvent<ReadyUpgrade<&'static [u8]>, (), Result, Self::Error>> {
        match self.state {
            State::Inactive { reported: true } => {
                return Poll::Pending; // nothing to do on this connection
            }
            State::Inactive { reported: false } => {
                self.state = State::Inactive { reported: true };
                return Poll::Ready(ConnectionHandlerEvent::Custom(Err(Failure::Unsupported)));
            }
            State::Active => {}
        }

        // Respond to inbound pings.
        if let Some(fut) = self.inbound.as_mut() {
            match fut.poll_unpin(cx) {
                Poll::Pending => {}
                Poll::Ready(Err(e)) => {
                    log::debug!("Inbound ping error: {:?}", e);
                    self.inbound = None;
                }
                Poll::Ready(Ok(stream)) => {
                    // A ping from a remote peer has been answered, wait for the next.
                    self.inbound = Some(recv_ping(stream).boxed());
                    return Poll::Ready(ConnectionHandlerEvent::Custom(Ok(Success::Pong)));
                }
            }
        }

        loop {
            // Check for outbound ping failures.
            if let Some(error) = self.pending_errors.pop_back() {
                log::debug!("Ping failure: {:?}", error);

                self.failures += 1;

                // Note: For backward-compatibility, with configured
                // `max_failures == 1`, the first failure is always "free"
                // and silent. This allows peers who still use a new substream
                // for each ping to have successful ping exchanges with peers
                // that use a single substream, since every successful ping
                // resets `failures` to `0`, while at the same time emitting
                // events only for `max_failures - 1` failures, as before.
                if self.failures > 1 || self.config.max_failures.get() > 1 {
                    if self.failures >= self.config.max_failures.get() {
                        log::debug!("Too many failures ({}). Closing connection.", self.failures);
                        return Poll::Ready(ConnectionHandlerEvent::Close(error));
                    }

                    return Poll::Ready(ConnectionHandlerEvent::Custom(Err(error)));
                }
            }

            // Continue outbound pings.
            match self.outbound.take() {
                Some(OutboundState::Ping(mut ping)) => match ping.poll_unpin(cx) {
                    Poll::Pending => {
                        if self.timer.poll_unpin(cx).is_ready() {
                            self.pending_errors.push_front(Failure::Timeout);
                        } else {
                            self.outbound = Some(OutboundState::Ping(ping));
                            break;
                        }
                    }
                    Poll::Ready(Ok((stream, rtt))) => {
                        self.failures = 0;
                        self.timer.reset(self.config.interval);
                        self.outbound = Some(OutboundState::Idle(stream));
                        return Poll::Ready(ConnectionHandlerEvent::Custom(Ok(Success::Ping {
                            rtt,
                        })));
                    }
                    Poll::Ready(Err(e)) => {
                        self.pending_errors
                            .push_front(Failure::Other { error: Box::new(e) });
                    }
                },
                Some(OutboundState::Idle(stream)) => match self.timer.poll_unpin(cx) {
                    Poll::Pending => {
                        self.outbound = Some(OutboundState::Idle(stream));
                        break;
                    }
                    Poll::Ready(()) => {
                        self.timer.reset(self.config.timeout);
                        self.outbound = Some(OutboundState::Ping(send_ping(stream).boxed()));
                    }
                },
                Some(OutboundState::OpenStream) => {
                    self.outbound = Some(OutboundState::OpenStream);
                    break;
                }
                None => {
                    self.outbound = Some(OutboundState::OpenStream);
                    let protocol = SubstreamProtocol::new(ReadyUpgrade::new(PROTOCOL_NAME), ())
                        .with_timeout(self.config.timeout);
                    return Poll::Ready(ConnectionHandlerEvent::OutboundSubstreamRequest {
                        protocol,
                    });
                }
            }
        }

        Poll::Pending
    }
}

type PingFuture =
    BoxFuture<'static, std::result::Result<(NegotiatedSubstream, Duration), io::Error>>;
type PongFuture = BoxFuture<'static, std::result::Result<NegotiatedSubstream, io::Error>>;

/// The current state w.r.t. outbound pings.
enum OutboundState {
    /// A new substream is being negotiated for the ping protocol.
    OpenStream,
    /// The substream is idle, waiting to send the next ping.
    Idle(NegotiatedSubstream),
    /// A ping is being sent and the response awaited.
    Ping(PingFuture),
}
