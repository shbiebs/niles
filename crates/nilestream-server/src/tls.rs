//! Transport security: negotiation, policy, and the boundary at which cryptography stops
//! being this thesis's problem (§6.12, Appendix D.7).
//!
//! # What is here and what is deliberately not
//!
//! This module implements everything about TLS that is *protocol-specific* — the
//! PostgreSQL `SSLRequest` exchange, the MySQL capability-flag upgrade, the policy that
//! decides whether a cleartext connection is allowed to proceed, and the downgrade
//! defences both protocols need. It implements **none** of the cryptography. The record
//! layer, the handshake, certificate-chain verification, and the cipher suites sit behind
//! the [`TlsProvider`] trait and are expected to be supplied by a vetted implementation —
//! `rustls` in the intended build.
//!
//! That division is not a shortcut, and the thesis should not present it as one. Writing
//! a TLS stack is the canonical example of work that must not be done from scratch: the
//! failure mode is silent, the attacker is adaptive, and the defects that matter
//! (Bleichenbacher oracles, timing leaks in MAC verification, state-machine confusion
//! allowing a `ClientKeyExchange` to be skipped) are exactly the ones a functional test
//! suite passes. A thesis whose contribution is *static checkability of financial
//! invariants* has nothing to gain and a great deal of credibility to lose by shipping
//! its own record layer. The honest engineering statement is therefore: the negotiation
//! is ours and is tested here; the cryptography is delegated, and the build that has no
//! provider **refuses to serve**, rather than falling back to cleartext.
//!
//! That last clause is the one worth defending. The obvious alternative — accept
//! cleartext when TLS is unavailable, log a warning — is how a database ends up serving a
//! ledger over a plaintext socket because a certificate expired at 3 a.m. and something
//! helpfully degraded. [`Policy::Require`] makes unavailability a startup failure. This
//! mirrors the choice made everywhere else in this system: an absence is reported
//! honestly rather than papered over, which is the same principle as `Hole` versus zero
//! in the absence lattice, applied to a socket instead of a key.
//!
//! # The two negotiations are not the same shape, and one of them is worse
//!
//! **PostgreSQL** is well designed here. The client sends an `SSLRequest` — eight bytes,
//! a length and a magic number, *before* the startup packet — and the server answers with
//! a single byte, `S` or `N`. Nothing else has been exchanged, so nothing is leaked by a
//! refusal, and the client's `sslmode` decides what to do with an `N`. The startup packet
//! carrying the user name and database is then sent inside the tunnel.
//!
//! **MySQL** is not. The server sends its handshake first, in cleartext, and the client
//! signals its intent to upgrade by setting `CLIENT_SSL` in the capability flags of a
//! *truncated* handshake response — the first 32 bytes, without user name or auth data —
//! then re-sends the full response inside the tunnel. Two consequences follow, and both
//! belong in the thesis rather than in a footnote:
//!
//! 1. The server's greeting, including its version banner and the auth-plugin name, is on
//!    the wire in the clear regardless. There is nothing a server can do about it; it is
//!    in the protocol.
//! 2. The upgrade is *client-asserted*. A server that merely honours the flag lets any
//!    client opt out of TLS by not setting it. Enforcement must therefore be a server-side
//!    check after reading the truncated response — [`MySqlNegotiation::decide`] does it —
//!    and this is a real historical vulnerability class, not a hypothetical: it is the
//!    shape of the "SSL/TLS downgrade" defect that affected several MySQL client
//!    libraries, where `--ssl` was a preference the server never verified.
//!
//! # `sslmode`, honestly
//!
//! `libpq` has six modes and only two of them are worth anything against an active
//! attacker. `require` encrypts but verifies nothing, so it stops passive capture and not
//! a man in the middle; `verify-ca` checks the chain; `verify-full` also checks that the
//! host name matches. We model all six because clients send all six, and we record in
//! [`ClientMode::authenticates_server`] which ones actually authenticate, so that a
//! deployment claiming "TLS is on" can be asked *which* TLS.

use std::fmt;

/// What the server requires of its clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Policy {
    /// TLS is not offered. A cleartext connection is served.
    ///
    /// Legitimate for a unix-socket-only deployment or a single-node test rig, and for
    /// nothing else. Named `Disabled` rather than `Optional` so that it appears in a
    /// configuration file as an explicit decision.
    Disabled,
    /// TLS is offered; a client that declines is still served in cleartext.
    ///
    /// This is the mode that provides the least security while appearing to provide some,
    /// because an attacker who can modify the stream simply strips the offer. It exists
    /// because migrations need it.
    Prefer,
    /// TLS is required. A client that declines is refused, and a server with no provider
    /// fails to start rather than serving cleartext.
    Require,
    /// TLS is required and the client must present a certificate the server can verify.
    ///
    /// The mutual case. For a ledger this is the defensible default in a datacentre,
    /// because it makes "who wrote this entry" a transport-level fact as well as an
    /// application-level claim, which matters when the audit obligation of §3.8 is to
    /// hold under a compromised application credential.
    RequireClientCert,
}

impl Policy {
    pub fn offers_tls(&self) -> bool {
        !matches!(self, Policy::Disabled)
    }

    pub fn permits_cleartext(&self) -> bool {
        matches!(self, Policy::Disabled | Policy::Prefer)
    }

    pub fn requires_client_cert(&self) -> bool {
        matches!(self, Policy::RequireClientCert)
    }
}

/// The client's stated preference, in `libpq`'s vocabulary.
///
/// We keep the names `libpq` uses rather than inventing better ones, because the value
/// arrives from a connection string the operator wrote and the diagnostic has to quote it
/// back to them recognisably.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientMode {
    Disable,
    Allow,
    Prefer,
    Require,
    VerifyCa,
    VerifyFull,
}

impl ClientMode {
    /// Whether the client will send an `SSLRequest` at all.
    pub fn attempts_tls(&self) -> bool {
        !matches!(self, ClientMode::Disable)
    }

    /// Whether the client will proceed in cleartext if the server answers `N`.
    pub fn falls_back_to_cleartext(&self) -> bool {
        matches!(
            self,
            ClientMode::Disable | ClientMode::Allow | ClientMode::Prefer
        )
    }

    /// **Whether this mode actually authenticates the server.**
    ///
    /// `require` does not. It encrypts against a peer it has not identified, which stops
    /// passive interception and does nothing whatever against an active one. This
    /// predicate exists so that an operator's claim to be "using TLS" can be checked
    /// rather than believed, and so that [`Negotiated::describe`] can say so.
    pub fn authenticates_server(&self) -> bool {
        matches!(self, ClientMode::VerifyCa | ClientMode::VerifyFull)
    }

    /// Whether the host name is checked against the certificate. Only `verify-full` does
    /// this, and without it a certificate issued for *any* host the CA will sign is
    /// accepted — which, for a public CA, is every host on the internet.
    pub fn checks_hostname(&self) -> bool {
        matches!(self, ClientMode::VerifyFull)
    }

    pub fn parse(s: &str) -> Option<ClientMode> {
        Some(match s.trim().to_ascii_lowercase().as_str() {
            "disable" => ClientMode::Disable,
            "allow" => ClientMode::Allow,
            "prefer" => ClientMode::Prefer,
            "require" => ClientMode::Require,
            "verify-ca" => ClientMode::VerifyCa,
            "verify-full" => ClientMode::VerifyFull,
            _ => return None,
        })
    }
}

impl fmt::Display for ClientMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ClientMode::Disable => "disable",
            ClientMode::Allow => "allow",
            ClientMode::Prefer => "prefer",
            ClientMode::Require => "require",
            ClientMode::VerifyCa => "verify-ca",
            ClientMode::VerifyFull => "verify-full",
        })
    }
}

/// Why a connection was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// The policy requires TLS and the client did not ask for it.
    CleartextRefused,
    /// The policy requires TLS and no provider is configured. This is a *server*
    /// misconfiguration surfaced at the connection rather than silently downgraded.
    NoProvider,
    /// Mutual TLS is required and the client presented nothing.
    ClientCertificateMissing,
    /// The handshake itself failed. The detail comes from the provider and is passed
    /// through verbatim, because a paraphrased TLS error is a debugging catastrophe.
    Handshake(String),
}

impl Refusal {
    /// The SQLSTATE this maps to on the PostgreSQL wire. `28000` is
    /// `invalid_authorization_specification`, which is what a real server sends when
    /// `hostssl` in `pg_hba.conf` rejects a non-TLS connection.
    pub fn sqlstate(&self) -> &'static str {
        match self {
            Refusal::CleartextRefused | Refusal::ClientCertificateMissing => "28000",
            // `08006` connection_failure. A handshake failure is not an authorization
            // decision and should not be reported as one, or an operator will spend the
            // evening checking credentials.
            Refusal::NoProvider | Refusal::Handshake(_) => "08006",
        }
    }

    pub fn message(&self) -> String {
        match self {
            Refusal::CleartextRefused => {
                "no pg_hba.conf entry for host: TLS is required by policy but the client \
                 did not request it (set sslmode=verify-full)"
                    .into()
            }
            Refusal::NoProvider => {
                "server requires TLS but no TLS provider is configured; refusing to serve \
                 in cleartext"
                    .into()
            }
            Refusal::ClientCertificateMissing => {
                "connection requires a valid client certificate".into()
            }
            Refusal::Handshake(d) => format!("TLS handshake failed: {d}"),
        }
    }
}

/// The outcome of a successful negotiation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Negotiated {
    pub encrypted: bool,
    /// Whether the *client* verified us. The server cannot observe this directly — it is
    /// inferred from the mode the client declared — so it is recorded as a claim rather
    /// than a fact, and [`Negotiated::describe`] says which.
    pub client_claims_verification: bool,
    pub client_cert_presented: bool,
    /// Protocol version, once a provider reports it. `None` in cleartext.
    pub version: Option<&'static str>,
}

impl Negotiated {
    pub fn cleartext() -> Self {
        Negotiated {
            encrypted: false,
            client_claims_verification: false,
            client_cert_presented: false,
            version: None,
        }
    }

    /// A one-line description for the audit log. The deliberate awkwardness of
    /// "encrypted, server identity unverified by the client" is the point: it is the true
    /// statement, and the comfortable phrasing ("TLS enabled") is not.
    pub fn describe(&self) -> String {
        if !self.encrypted {
            return "cleartext".into();
        }
        let v = self.version.unwrap_or("TLS");
        match (self.client_claims_verification, self.client_cert_presented) {
            (true, true) => format!("{v}, mutually authenticated"),
            (true, false) => format!("{v}, server identity verified by client"),
            (false, true) => {
                format!("{v}, client authenticated, server identity unverified by client")
            }
            (false, false) => {
                format!("{v}, encrypted, server identity unverified by the client")
            }
        }
    }
}

/// The cryptographic boundary.
///
/// An implementation of this trait wraps a byte stream in a TLS session. Nothing in this
/// crate implements it; the intended production implementation delegates to `rustls`.
/// Keeping it a trait rather than a direct dependency has one further benefit for the
/// thesis: the negotiation state machines below are testable without a certificate, a
/// clock, or a network, which is why they have tests at all.
pub trait TlsProvider {
    /// A human-readable name for the log line, e.g. `"rustls 0.23"`.
    fn name(&self) -> &str;
    /// Perform the handshake over an already-negotiated upgrade.
    fn accept(&self) -> Result<Negotiated, Refusal>;
    /// Whether this provider can enforce client certificates.
    fn supports_client_auth(&self) -> bool {
        false
    }
}

/// The provider used when the build has no TLS.
///
/// It does not silently do nothing: it fails every `accept`, so that a `Require` policy
/// paired with this provider produces a refusal at connection time and a startup check
/// produces one before that.
pub struct NoProvider;

impl TlsProvider for NoProvider {
    fn name(&self) -> &str {
        "none (no TLS provider compiled in)"
    }
    fn accept(&self) -> Result<Negotiated, Refusal> {
        Err(Refusal::NoProvider)
    }
}

/// The server's transport-security configuration.
pub struct TlsConfig {
    pub policy: Policy,
    pub provider: Box<dyn TlsProvider>,
}

impl TlsConfig {
    pub fn insecure() -> Self {
        TlsConfig {
            policy: Policy::Disabled,
            provider: Box::new(NoProvider),
        }
    }

    pub fn with_policy(policy: Policy, provider: Box<dyn TlsProvider>) -> Self {
        TlsConfig { policy, provider }
    }

    /// The check that runs at boot, before the listener binds.
    ///
    /// Failing here rather than at the first connection is the whole design: a server
    /// that starts and then refuses every connection is indistinguishable, from the
    /// outside, from a server that is up. One that refuses to start is not.
    pub fn preflight(&self) -> Result<(), Refusal> {
        if !self.policy.offers_tls() {
            return Ok(());
        }
        if self.provider.accept().is_err() && !self.policy.permits_cleartext() {
            return Err(Refusal::NoProvider);
        }
        if self.policy.requires_client_cert() && !self.provider.supports_client_auth() {
            return Err(Refusal::NoProvider);
        }
        Ok(())
    }
}

// ── PostgreSQL negotiation ───────────────────────────────────────────────────────────

/// The single byte the server sends in reply to an `SSLRequest`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SslReply {
    /// `S` — proceed with the handshake.
    Willing,
    /// `N` — continue in cleartext, or disconnect, as the client's mode dictates.
    Unwilling,
}

impl SslReply {
    pub fn byte(&self) -> u8 {
        match self {
            SslReply::Willing => b'S',
            SslReply::Unwilling => b'N',
        }
    }
}

/// What the server should do with a connection, in PostgreSQL's ordering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PgStep {
    /// Send this byte, then perform the handshake, then read the startup packet.
    Upgrade(SslReply),
    /// Send `N`, then read the startup packet in cleartext.
    Cleartext(SslReply),
    /// Refuse. An `ErrorResponse` carries the reason; the connection then closes.
    Refuse(Refusal),
}

/// The PostgreSQL side of the negotiation.
pub struct PgNegotiation<'a> {
    pub config: &'a TlsConfig,
}

impl<'a> PgNegotiation<'a> {
    pub fn new(config: &'a TlsConfig) -> Self {
        PgNegotiation { config }
    }

    /// Decide what to do, given whether the client sent an `SSLRequest`.
    ///
    /// The ordering constraint that makes this correct: the reply byte goes out *before*
    /// any handshake and before the startup packet is read, so a refusal reveals nothing
    /// beyond the fact of the refusal. There is no user name on the wire yet.
    pub fn decide(&self, client_requested_ssl: bool) -> PgStep {
        match (self.config.policy, client_requested_ssl) {
            // The client did not ask, and policy allows cleartext.
            (p, false) if p.permits_cleartext() => PgStep::Cleartext(SslReply::Unwilling),
            // The client did not ask, and policy does not allow that.
            (_, false) => PgStep::Refuse(Refusal::CleartextRefused),
            // The client asked and we do not offer it. Answering `N` is correct and
            // sufficient: a `require` client will disconnect on its own, which is the
            // client's decision to make and not ours to pre-empt.
            (Policy::Disabled, true) => PgStep::Cleartext(SslReply::Unwilling),
            // The client asked and we offer it.
            (_, true) => match self.config.provider.accept() {
                Ok(_) => PgStep::Upgrade(SslReply::Willing),
                // A provider that cannot handshake and a policy that permits cleartext:
                // answer `N` and let the client choose. Under `Require`, refuse.
                Err(e) => {
                    if self.config.policy.permits_cleartext() {
                        PgStep::Cleartext(SslReply::Unwilling)
                    } else {
                        PgStep::Refuse(e)
                    }
                }
            },
        }
    }

    /// The eight-byte `SSLRequest` a client sends, for tests and for the client side.
    ///
    /// Length 8, then the magic `80877103` = `(1234 << 16) | 5679`. It is deliberately
    /// not a valid protocol version, which is how a server distinguishes it from a
    /// startup packet without a mode flag.
    pub fn ssl_request_bytes() -> [u8; 8] {
        let mut b = [0u8; 8];
        b[..4].copy_from_slice(&8i32.to_be_bytes());
        b[4..].copy_from_slice(&80_877_103i32.to_be_bytes());
        b
    }
}

// ── MySQL negotiation ────────────────────────────────────────────────────────────────

/// What the server should do with a MySQL connection after reading the truncated
/// handshake response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MySqlStep {
    /// The client set `CLIENT_SSL`; perform the handshake, then read the full response.
    Upgrade,
    /// Continue in cleartext.
    Cleartext,
    /// Refuse with an `ERR` packet.
    Refuse(Refusal),
}

/// The MySQL side.
///
/// The asymmetry with PostgreSQL is worth restating where the code is: by the time this
/// runs, the server greeting has *already* gone out in cleartext. Only the credentials
/// are protected by the upgrade, never the banner.
pub struct MySqlNegotiation<'a> {
    pub config: &'a TlsConfig,
}

impl<'a> MySqlNegotiation<'a> {
    pub fn new(config: &'a TlsConfig) -> Self {
        MySqlNegotiation { config }
    }

    /// `CLIENT_SSL`, as asserted by the client in the low 32 bits of its capability flags.
    pub const CLIENT_SSL: u32 = 0x0000_0800;

    /// Decide, given the client's asserted capability flags.
    ///
    /// The security-relevant line is the `(_, false)` arm under a non-cleartext policy:
    /// **the server checks, rather than trusting the client's flag.** A server that only
    /// branches on `CLIENT_SSL` being set has made TLS opt-in for the attacker.
    pub fn decide(&self, client_flags: u32) -> MySqlStep {
        let asked = client_flags & Self::CLIENT_SSL != 0;
        match (self.config.policy, asked) {
            (p, false) if p.permits_cleartext() => MySqlStep::Cleartext,
            (_, false) => MySqlStep::Refuse(Refusal::CleartextRefused),
            (Policy::Disabled, true) => MySqlStep::Cleartext,
            (_, true) => match self.config.provider.accept() {
                Ok(_) => MySqlStep::Upgrade,
                Err(e) => {
                    if self.config.policy.permits_cleartext() {
                        MySqlStep::Cleartext
                    } else {
                        MySqlStep::Refuse(e)
                    }
                }
            },
        }
    }

    /// The capability flags the server advertises in its greeting.
    ///
    /// Advertising `CLIENT_SSL` is what tells a client an upgrade is available; a server
    /// that requires TLS must still advertise it, or no client will offer.
    pub fn server_capabilities(&self, base: u32) -> u32 {
        if self.config.policy.offers_tls() {
            base | Self::CLIENT_SSL
        } else {
            base & !Self::CLIENT_SSL
        }
    }

    /// The truncated handshake response is exactly 32 bytes on the wire (4 capability, 4
    /// max-packet, 1 charset, 23 reserved) with no user name. A longer payload after a
    /// `CLIENT_SSL` flag means the client sent credentials in the clear, which is a
    /// protocol violation the server should notice rather than accept.
    pub fn truncated_response_is_wellformed(payload: &[u8]) -> bool {
        payload.len() == 32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A provider that always succeeds, standing in for `rustls` so the negotiation logic
    /// can be tested without cryptography.
    struct FakeTls {
        client_auth: bool,
    }

    impl TlsProvider for FakeTls {
        fn name(&self) -> &str {
            "fake (test only)"
        }
        fn accept(&self) -> Result<Negotiated, Refusal> {
            Ok(Negotiated {
                encrypted: true,
                client_claims_verification: false,
                client_cert_presented: self.client_auth,
                version: Some("TLSv1.3"),
            })
        }
        fn supports_client_auth(&self) -> bool {
            self.client_auth
        }
    }

    fn require_with_tls() -> TlsConfig {
        TlsConfig::with_policy(Policy::Require, Box::new(FakeTls { client_auth: false }))
    }

    // ── the central safety property ──────────────────────────────────────────────────

    #[test]
    fn require_without_a_provider_fails_at_startup_rather_than_serving_cleartext() {
        // The whole point of the module. A build with no TLS and a policy demanding TLS
        // must not start. If this test ever fails, a ledger is being served in the clear.
        let c = TlsConfig::with_policy(Policy::Require, Box::new(NoProvider));
        assert_eq!(c.preflight(), Err(Refusal::NoProvider));
    }

    #[test]
    fn and_if_it_somehow_started_it_still_refuses_every_connection() {
        // Defence in depth: preflight is a check, not a guarantee, because a provider can
        // stop working after boot (an expired certificate, a revoked key). The
        // per-connection path must reach the same conclusion independently.
        let c = TlsConfig::with_policy(Policy::Require, Box::new(NoProvider));
        assert_eq!(
            PgNegotiation::new(&c).decide(true),
            PgStep::Refuse(Refusal::NoProvider)
        );
        assert_eq!(
            MySqlNegotiation::new(&c).decide(MySqlNegotiation::CLIENT_SSL),
            MySqlStep::Refuse(Refusal::NoProvider)
        );
    }

    #[test]
    fn a_cleartext_client_is_refused_under_require_on_both_protocols() {
        let c = require_with_tls();
        assert_eq!(
            PgNegotiation::new(&c).decide(false),
            PgStep::Refuse(Refusal::CleartextRefused)
        );
        assert_eq!(
            MySqlNegotiation::new(&c).decide(0),
            MySqlStep::Refuse(Refusal::CleartextRefused)
        );
    }

    #[test]
    fn mysql_enforcement_does_not_trust_the_clients_flag() {
        // The downgrade defence. A client that simply omits CLIENT_SSL must be refused,
        // not quietly served. This is the arm that a naive implementation gets wrong,
        // because the naive implementation branches only on the flag being *set*.
        let c = require_with_tls();
        let n = MySqlNegotiation::new(&c);
        assert_eq!(n.decide(0), MySqlStep::Refuse(Refusal::CleartextRefused));
        assert_eq!(
            n.decide(0xFFFF_F7FF),
            MySqlStep::Refuse(Refusal::CleartextRefused)
        );
        assert_eq!(n.decide(MySqlNegotiation::CLIENT_SSL), MySqlStep::Upgrade);
    }

    // ── the ordinary paths ───────────────────────────────────────────────────────────

    #[test]
    fn prefer_serves_both_kinds_of_client() {
        let c = TlsConfig::with_policy(Policy::Prefer, Box::new(FakeTls { client_auth: false }));
        assert_eq!(
            PgNegotiation::new(&c).decide(true),
            PgStep::Upgrade(SslReply::Willing)
        );
        assert_eq!(
            PgNegotiation::new(&c).decide(false),
            PgStep::Cleartext(SslReply::Unwilling)
        );
    }

    #[test]
    fn disabled_answers_n_to_a_request_rather_than_erroring() {
        // Answering `N` is the protocol-correct refusal and lets the client's own
        // sslmode decide. Sending an ErrorResponse here would break `sslmode=prefer`
        // clients that would otherwise have connected fine.
        let c = TlsConfig::insecure();
        assert_eq!(
            PgNegotiation::new(&c).decide(true),
            PgStep::Cleartext(SslReply::Unwilling)
        );
        assert_eq!(c.preflight(), Ok(()));
    }

    #[test]
    fn mutual_tls_requires_a_provider_that_can_actually_do_it() {
        let no_auth = TlsConfig::with_policy(
            Policy::RequireClientCert,
            Box::new(FakeTls { client_auth: false }),
        );
        assert_eq!(no_auth.preflight(), Err(Refusal::NoProvider));

        let with_auth = TlsConfig::with_policy(
            Policy::RequireClientCert,
            Box::new(FakeTls { client_auth: true }),
        );
        assert_eq!(with_auth.preflight(), Ok(()));
    }

    // ── wire-level details ───────────────────────────────────────────────────────────

    #[test]
    fn the_ssl_request_magic_number_is_the_real_one() {
        // 80877103 = (1234 << 16) | 5679. Getting this wrong means every psql connection
        // with sslmode=prefer is misparsed as a startup packet with a bizarre version.
        let b = PgNegotiation::ssl_request_bytes();
        assert_eq!(i32::from_be_bytes([b[0], b[1], b[2], b[3]]), 8);
        assert_eq!(i32::from_be_bytes([b[4], b[5], b[6], b[7]]), 80_877_103);
        assert_eq!(80_877_103, (1234i32 << 16) | 5679);
    }

    #[test]
    fn the_reply_is_one_byte_and_it_is_s_or_n() {
        assert_eq!(SslReply::Willing.byte(), b'S');
        assert_eq!(SslReply::Unwilling.byte(), b'N');
    }

    #[test]
    fn the_server_advertises_client_ssl_exactly_when_it_offers_tls() {
        let on = TlsConfig::with_policy(Policy::Require, Box::new(FakeTls { client_auth: false }));
        let off = TlsConfig::insecure();
        let base = 0x000F_A68D;
        assert_ne!(
            MySqlNegotiation::new(&on).server_capabilities(base) & MySqlNegotiation::CLIENT_SSL,
            0
        );
        assert_eq!(
            MySqlNegotiation::new(&off).server_capabilities(base) & MySqlNegotiation::CLIENT_SSL,
            0
        );
        // Every other advertised capability survives the masking, or the greeting changes
        // meaning when TLS is toggled.
        assert_eq!(
            MySqlNegotiation::new(&on).server_capabilities(base) & !MySqlNegotiation::CLIENT_SSL,
            base & !MySqlNegotiation::CLIENT_SSL
        );
    }

    #[test]
    fn a_truncated_mysql_response_is_exactly_thirty_two_bytes() {
        // Longer means the client attached credentials to the packet that precedes the
        // handshake, i.e. sent them in the clear.
        assert!(MySqlNegotiation::truncated_response_is_wellformed(
            &[0u8; 32]
        ));
        assert!(!MySqlNegotiation::truncated_response_is_wellformed(
            &[0u8; 64]
        ));
        assert!(!MySqlNegotiation::truncated_response_is_wellformed(
            &[0u8; 20]
        ));
    }

    // ── the honesty properties ───────────────────────────────────────────────────────

    #[test]
    fn require_does_not_authenticate_the_server_and_the_type_says_so() {
        // The claim this module refuses to let a deployment make. `sslmode=require`
        // encrypts to an unidentified peer.
        assert!(!ClientMode::Require.authenticates_server());
        assert!(ClientMode::VerifyCa.authenticates_server());
        assert!(ClientMode::VerifyFull.authenticates_server());
        // And only verify-full checks the name on the certificate.
        assert!(!ClientMode::VerifyCa.checks_hostname());
        assert!(ClientMode::VerifyFull.checks_hostname());
    }

    #[test]
    fn the_audit_line_states_what_was_actually_achieved() {
        let bare = Negotiated {
            encrypted: true,
            client_claims_verification: false,
            client_cert_presented: false,
            version: Some("TLSv1.3"),
        };
        assert_eq!(
            bare.describe(),
            "TLSv1.3, encrypted, server identity unverified by the client"
        );

        let mutual = Negotiated {
            encrypted: true,
            client_claims_verification: true,
            client_cert_presented: true,
            version: Some("TLSv1.3"),
        };
        assert_eq!(mutual.describe(), "TLSv1.3, mutually authenticated");

        assert_eq!(Negotiated::cleartext().describe(), "cleartext");
    }

    #[test]
    fn every_libpq_sslmode_parses_and_round_trips() {
        for s in [
            "disable",
            "allow",
            "prefer",
            "require",
            "verify-ca",
            "verify-full",
        ] {
            let m = ClientMode::parse(s).unwrap_or_else(|| panic!("{s} should parse"));
            assert_eq!(m.to_string(), s);
        }
        assert_eq!(
            ClientMode::parse("VERIFY-FULL"),
            Some(ClientMode::VerifyFull)
        );
        assert_eq!(
            ClientMode::parse("yes"),
            None,
            "an unknown mode must not default to something"
        );
    }

    #[test]
    fn which_modes_fall_back_to_cleartext_is_pinned() {
        // A regression here silently changes what happens when a server answers `N`.
        assert!(ClientMode::Prefer.falls_back_to_cleartext());
        assert!(ClientMode::Allow.falls_back_to_cleartext());
        assert!(!ClientMode::Require.falls_back_to_cleartext());
        assert!(!ClientMode::VerifyFull.falls_back_to_cleartext());
        assert!(!ClientMode::Disable.attempts_tls());
    }

    #[test]
    fn a_handshake_failure_is_a_connection_error_and_not_an_auth_error() {
        // Mapping a TLS failure to 28000 sends the operator to check credentials for an
        // hour. It is a connection failure and 08006 says so.
        assert_eq!(
            Refusal::Handshake("bad certificate".into()).sqlstate(),
            "08006"
        );
        assert_eq!(Refusal::NoProvider.sqlstate(), "08006");
        assert_eq!(Refusal::CleartextRefused.sqlstate(), "28000");
        assert!(
            Refusal::Handshake("expired".into())
                .message()
                .contains("expired"),
            "the provider's detail must survive verbatim"
        );
    }

    #[test]
    fn no_provider_is_a_refusal_and_never_a_silent_success() {
        // The failure mode this whole module exists to prevent, asserted at the smallest
        // possible scope.
        assert_eq!(NoProvider.accept(), Err(Refusal::NoProvider));
        assert!(!NoProvider.supports_client_auth());
        assert!(NoProvider.name().contains("no TLS"));
    }
}
