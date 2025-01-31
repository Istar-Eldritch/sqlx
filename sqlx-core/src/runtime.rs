#[cfg(not(any(feature = "runtime-tokio", feature = "runtime-async-std")))]
compile_error!("one of 'runtime-async-std' or 'runtime-tokio' features must be enabled");

#[cfg(all(feature = "runtime-tokio", feature = "runtime-async-std"))]
compile_error!("only one of 'runtime-async-std' or 'runtime-tokio' features must be enabled");

#[cfg(feature = "runtime-async-std")]
pub(crate) use async_std::{
    future::timeout,
    io::prelude::ReadExt as AsyncReadExt,
    io::{Read as AsyncRead, Write as AsyncWrite},
    net::TcpStream,
    task::sleep,
    task::spawn,
};

#[cfg(feature = "runtime-tokio")]
pub(crate) use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite},
    net::TcpStream,
    task::spawn,
    time::delay_for as sleep,
    time::timeout,
};
