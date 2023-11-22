use std::convert::Infallible;
use std::io;
use std::net;
use std::net::IpAddr;
use std::net::ToSocketAddrs;

use octets::Octets;


// A specialized [`Result`] type for proxy operations.
pub type Result<T> = std::result::Result<T, Error>;

// TODO: Implement detailed error description when needed.
// #[derive(Debug, Clone)]
// struct ErrorData {
//   name: String,
//   description: String,
//   extra_parameters: HashMap<String, String>,
//   http_status_code: u16,
//   response_only_generated_by_intermediaries: bool,
//   reference: String,
// }

// A RFC 9209 Proxy Error
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
  DnsTimeout,
  DnsError,
  DestinationNotFound,
  DestinationUnavailable,
  DestinationIpProhibited,
  DestinationIpUnroutable,
  ConnectionRefused,
  ConnectionTerminated,
  ConnectionReadTimeout,
  ConnectionWriteTimeout,
  ConnectionLimitReached,
  TlsProtocolError,
  TlsCertificateError,
  TlsAlertReceived,
  HttpRequestError(u16),
  HttpRequestDenied,
  HttpResponseIncomplete,
  HttpResponseHeaderSectionSize,
  HttpResponseHeaderSize,
  HttpResponseBodySize,
  HttpResponseTrailerSectionSize,
  HttpResponseTrailerSize,
  HttpResponseTransferCoding,
  HttpResponseContentCoding,
  HttpResponseTimeout,
  HttpUpgradeFailed,
  HttpProtocolError,
  ProxyInternalResponse(u16),
  ProxyInternalError,
  ProxyConfigurationError,
  ProxyLoopDetected,
}

impl Error {
  fn to_status_code(self) -> u16 {
    match self {
      Error::DnsTimeout => 504,
      Error::DnsError => 502,
      Error::DestinationNotFound => 500,
      Error::DestinationUnavailable => 503,
      Error::DestinationIpProhibited => 502,
      Error::DestinationIpUnroutable => 502,
      Error::ConnectionRefused => 502,
      Error::ConnectionTerminated => 502,
      Error::ConnectionReadTimeout => 504,
      Error::ConnectionWriteTimeout => 504,
      Error::ConnectionLimitReached => 503,
      Error::TlsProtocolError => 502,
      Error::TlsCertificateError => 502,
      Error::TlsAlertReceived => 502,
      Error::HttpRequestError(status_code) => status_code,
      Error::HttpRequestDenied => 403,
      Error::HttpResponseIncomplete => 502,
      Error::HttpResponseHeaderSectionSize => 502,
      Error::HttpResponseHeaderSize => 502,
      Error::HttpResponseBodySize => 502,
      Error::HttpResponseTrailerSectionSize => 502,
      Error::HttpResponseTrailerSize => 502,
      Error::HttpResponseTransferCoding => 502,
      Error::HttpResponseContentCoding => 502,
      Error::HttpResponseTimeout => 504,
      Error::HttpUpgradeFailed => 502,
      Error::HttpProtocolError => 502,
      Error::ProxyInternalResponse(status_code) => status_code,
      Error::ProxyInternalError => 500,
      Error::ProxyConfigurationError => 500,
      Error::ProxyLoopDetected => 502,
    }
  }
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{self:?}")
  }
}

impl std::convert::From<std::io::Error> for Error {
  fn from(err: std::io::Error) -> Self {
    error!("IO Error {:?}", err);
    Error::ProxyInternalError
  }
}

impl std::convert::From<std::net::AddrParseError> for Error {
  fn from(err: std::net::AddrParseError) -> Self {
    error!("Addr Parse Error {:?}", err);
    Error::ProxyInternalError
  }
}


pub struct Http3DgramProxy {
    socket: mio::net::UdpSocket,
    pub target: net::SocketAddr,
    pub context_id: u64,
}

impl Http3DgramProxy {
  pub fn with_host_port(host: &str, port: &str) -> Result<Self> {
    let port = port.parse::<u16>().map_err(|_| Error::DestinationNotFound)?;

    Self::with_sock_addr(format!("{host}:{port}")
      .to_socket_addrs()?
      .next()
      .ok_or(Error::DnsError)?
    )
  }

  pub fn with_addr_port(addr: IpAddr, port: u16) -> Result<Self> {
    Self::with_sock_addr(net::SocketAddr::new(addr, port))
  }

  pub fn with_sock_addr(target: net::SocketAddr) -> Result<Self> {
    // TODO: make the bind address configurable.
    let socket: mio::net::UdpSocket = mio::net::UdpSocket::bind("0.0.0.0:0".parse()?)?;

    if let Err(e) = mio::net::UdpSocket::connect(&socket,target){
      if e.kind() == std::io::ErrorKind::WouldBlock {
        trace!("connect() would block");
        //TODO: Retry bind later
      }
      error!("connect() failed: {:?}", e);
      return Err(Error::ConnectionRefused);
    }
    
    // Context ID fixed to 0 right now.
    let context_id = 0;

    Ok(Self { socket, target, context_id })
  }

  pub fn send(&self, buf: &[u8]) -> std::io::Result<usize> {
    self.socket.send(buf)
  }

  pub fn recv(&self, buf: &mut [u8]) -> std::io::Result<usize> {
    self.socket.recv(buf)
  }
}