use tokio::net::TcpStream as UnencryptedTcpStream;
use tokio_native_tls::TlsStream as EncryptedTcpStream;
use tokio_native_tls::{TlsAcceptor};
use native_tls::Error as TlsError;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::replies::Reply;
use bytes::BufMut;

pub enum TcpStream {
    Unencrypted(UnencryptedTcpStream),
    Encrypted(EncryptedTcpStream<UnencryptedTcpStream>),
}

impl TcpStream {
    pub async fn send_reply(&mut self, reply: Reply) -> std::result::Result<(), std::io::Error> {
        match self {
            TcpStream::Unencrypted(s) => s.write_all(reply.to_string().as_bytes()).await,
            TcpStream::Encrypted(s) => s.write_all(reply.to_string().as_bytes()).await,
        }
    }

    pub async fn read_buf<'a, B>(&'a mut self, buf: &'a mut B) -> std::result::Result<usize, std::io::Error> where
        Self: Sized + Unpin,
        B: BufMut,  {
        match self {
            TcpStream::Unencrypted(s) => s.read_buf(buf).await,
            TcpStream::Encrypted(s) => s.read_buf(buf).await,
        }
    }

    pub async fn shutdown(&mut self) -> std::result::Result<(), std::io::Error> {
        match self {
            TcpStream::Unencrypted(s) => s.shutdown().await,
            TcpStream::Encrypted(s) => s.shutdown().await,
        }
    }

    pub async fn accept(self, tls_acceptor: &TlsAcceptor) -> Result<TcpStream, TlsError> {
        match self {
            TcpStream::Unencrypted(s) => {
                match tls_acceptor.accept(s).await {
                    Ok(s) => {
                        Ok(TcpStream::Encrypted(s))
                    },
                    Err(e) => {
                        Err(e)
                    }
                }
            },
            TcpStream::Encrypted(s) => Ok(TcpStream::Encrypted(s))
        }
    }
}