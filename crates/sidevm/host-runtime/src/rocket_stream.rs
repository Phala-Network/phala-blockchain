use std::{
    collections::VecDeque,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use rocket::{
    data::{ByteUnit, DataStream},
    http::{Header, Status},
    request::{FromRequest, Outcome},
    response::Responder,
    Data, Request,
};
use sidevm_env::messages::{HttpHead, HttpResponseHead};
use tokio::{
    io::{AsyncRead, AsyncWrite, DuplexStream, ReadBuf},
    sync::oneshot::Receiver,
};

use crate::IncomingHttpRequest;

pub struct DataHttpHead(pub HttpHead);

pub struct BridgedResponse<'r> {
    head: HttpResponseHead,
    body_stream: Bridge<'r>,
}

impl<'r> BridgedResponse<'r> {
    pub fn new(head: HttpResponseHead, body_stream: Bridge<'r>) -> Self {
        Self { head, body_stream }
    }
}

pub struct Bridge<'r> {
    input_body: DataStream<'r>,
    output_stream: DuplexStream,
    cached_data: VecDeque<u8>,
}

impl<'r> Future for Bridge<'r> {
    type Output = std::io::Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            while !self.cached_data.is_empty() {
                let me = &mut *self;
                match Pin::new(&mut me.output_stream).poll_write(cx, me.cached_data.as_slices().0) {
                    Poll::Ready(Ok(sz)) => {
                        if sz == 0 {
                            return Poll::Ready(Err(std::io::Error::new(
                                std::io::ErrorKind::WriteZero,
                                "failed to write",
                            )));
                        }
                        let _ = me.cached_data.drain(..sz);
                    }
                    Poll::Ready(Err(err)) => return Poll::Ready(Err(err)),
                    Poll::Pending => return Poll::Pending,
                }
            }

            let mut input = [0u8; 256];
            let mut input_buf = ReadBuf::new(&mut input);
            match Pin::new(&mut self.input_body).poll_read(cx, &mut input_buf) {
                Poll::Ready(Ok(_)) => {
                    let sz = input_buf.filled().len();
                    if sz == 0 {
                        return Poll::Ready(Ok(()));
                    }
                    self.cached_data.extend(&input[..sz]);
                    continue;
                }
                Poll::Ready(Err(err)) => return Poll::Ready(Err(err)),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

impl<'r> AsyncRead for Bridge<'r> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        if let Poll::Ready(Err(err)) = Pin::new(&mut *self).poll(cx) {
            return Poll::Ready(Err(err));
        }
        Pin::new(&mut self.output_stream).poll_read(cx, buf)
    }
}

impl<'r> Responder<'r, 'r> for BridgedResponse<'r> {
    fn respond_to(self, _req: &'r Request<'_>) -> rocket::response::Result<'r> {
        let mut builder = rocket::response::Response::build();
        builder.status(Status::new(self.head.status));
        for (name, value) in self.head.headers.into_iter() {
            builder.header_adjoin(Header::new(name, value));
        }
        builder.streamed_body(self.body_stream);
        builder.upgrade("h2c");
        Ok(builder.finalize())
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for DataHttpHead {
    type Error = &'static str;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let method = req.method().to_string();
        let uri = req.uri().to_string();
        let headers = req
            .headers()
            .iter()
            .map(|header| (header.name.to_string(), header.value.to_string()))
            .collect();
        Outcome::Success(DataHttpHead(HttpHead {
            method,
            uri,
            headers,
        }))
    }
}

pub fn bridge(
    head: HttpHead,
    body: Data,
) -> (
    Bridge,
    IncomingHttpRequest,
    Receiver<anyhow::Result<HttpResponseHead>>,
) {
    let (response_tx, response_rx) = tokio::sync::oneshot::channel();
    let (stream0, stream1) = tokio::io::duplex(1024);
    let bridge = Bridge {
        input_body: body.open(ByteUnit::max_value()),
        output_stream: stream0,
        cached_data: Default::default(),
    };
    let command = IncomingHttpRequest {
        head,
        body_stream: stream1,
        response_tx,
    };
    (bridge, command, response_rx)
}
