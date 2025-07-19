mod crypt_writer;

use crypt_writer::CryptWriter;
use futures::prelude::*;
use pin_project::pin_project;
use salsa20::{
    Salsa20, XSalsa20,
    cipher::{KeyIvInit, StreamCipher},
};
use sha3::Shake128;
use std::error;
use std::io;
use std::{
    fmt,
    pin::Pin,
    task::{Context, Poll},
};

const NONCE_SIZE: usize = 32;
const WRITE_BUFFER_SIZE: usize = 1024;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct PreSharedKey([u8; KEY_SIZE]);

impl PreSharedKey {
    /// Create a new pre shared key from raw bytes
    pub fn new(data: [u8; KEY_SIZE]) -> Self {
        Self(data)
    }

    /// Compute PreSharedKey fingerprint identical to the go-libp2p fingerprint.
    /// The computation of the fingerprint is not specified in the spec.
    ///
    /// This provides a way to check that private keys are properly configured
    /// without dumping the key itself to the console.
    pub fn fingerprint(&self) -> Fingerprint {
        use std::io::{Read, Write};
        let mut enc = [0u8; 64];
        let nonce: [u8; 8] = *b"finprint";
        let mut out = [0u8; 16];
        let mut cipher = Salsa20::new(&Into::into(self.0), &nonce.into());
        cipher.apply_keystream(&mut enc);
        let mut hasher = Shake128::default();
        hasher.write_all(&enc).expect("shake128 failed");
        hasher
            .finalize_xof()
            .read_exact(&mut out)
            .expect("shake128 failed");
        Fingerprint(out)
    }
}

/// Private network configuration
#[derive(Debug, Clone)]
pub struct Transport {
    /// the PreSharedKey to use for encryption
    psk: PreSharedKey,
}

impl Transport {
    pub fn new(psk: PreSharedKey) -> Self {
        Self { psk }
    }

    /// upgrade a connection to use pre shared key encryption.
    ///
    /// the upgrade works by both sides exchanging 24 byte nonces and then encrypting
    /// subsequent traffic with XSalsa20
    pub async fn handshake<TSocket>(self, mut socket: TSocket) -> Result<Output<TSocket>, Error>
    where
        TSocket: AsyncRead + AsyncWrite + Send + Unpin + 'static,
    {
        tracing::trace!("exchanging nonces");
        let mut local_nonce = [0u8; NONCE_SIZE];
        let mut remote_nonce = [0u8; NONCE_SIZE];
        rand::fill(&mut local_nonce);
        socket
            .write_all(&local_nonce)
            .await
            .map_err(Error::HandshakeError)?;
        socket.flush().await?;
        socket
            .read_exact(&mut remote_nonce)
            .await
            .map_err(Error::HandshakeError)?;
        tracing::trace!("setting up ciphers");
        let write_cipher = XSalsa20::new(&self.psk.0.into(), &local_nonce);
        let read_cipher = XSalsa20::new(&self.psk.0.into(), &remote_nonce);
        Ok(Output::new(socket, write_cipher, read_cipher))
    }
}

/// The result of a handshake. This implements AsyncRead and AsyncWrite and can therefore
/// be used as base for additional upgrades.
#[pin_project]
pub struct Output<S> {
    #[pin]
    inner: CryptWriter<S>,
    read_cipher: XSalsa20,
}

impl<S: AsyncRead + AsyncWrite> Output<S> {
    fn new(inner: S, write_cipher: XSalsa20, read_cipher: XSalsa20) -> Self {
        Self {
            inner: CryptWriter::with_capacity(WRITE_BUFFER_SIZE, inner, write_cipher),
            read_cipher,
        }
    }
}

impl<S: AsyncRead + AsyncWrite> AsyncRead for Output<S> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, io::Error>> {
        let this = self.project();
        let result = this.inner.get_pin_mut().poll_read(cx, buf);
        if let Poll::Ready(Ok(size)) = &result {
            tracing::trace!(bytes=%size, "read bytes");
            this.read_cipher.apply_keystream(&mut buf[..*size]);
            tracing::trace!(bytes=%size, "decrypted bytes");
        }
        result
    }
}

impl<S: AsyncRead + AsyncWrite> AsyncWrite for Output<S> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        self.project().inner.poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        self.project().inner.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        self.project().inner.poll_close(cx)
    }
}

type IoError = io::Error;

/// Error when writing or reading private swarms
#[derive(Debug)]
pub enum Error {
    /// Error during handshake.
    HandshakeError(io::Error),
    /// I/O error.
    IoError(io::Error),
}

impl From<io::Error> for Error {
    #[inline]
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}

impl error::Error for Error {
    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            Error::HandshakeError(ref err) => Some(err),
            Error::IoError(ref err) => Some(err),
        }
    }
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Error::HandshakeError(e) => write!(f, "Handshake error: {e}"),
            Error::IoError(e) => write!(f, "I/O error: {e}"),
        }
    }
}
