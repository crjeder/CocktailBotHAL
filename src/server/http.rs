// src/server/http.rs
//
// Minimal HTTP/1.1 request parser and JSON response writer for
// embedded-io-async sockets. Replaces the non-existent `http_smol`
// crate that was referenced in earlier code.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use embedded_io_async::{Read, Write};

use crate::hal::ErrorInfo;

/// Maximum header size accepted (4 KiB).
const MAX_HEADER_BYTES: usize = 4096;

// =========================================================================
// Types
// =========================================================================

/// A parsed HTTP/1.1 request.
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub body: Vec<u8>,
}

/// Errors that can occur during HTTP I/O.
#[derive(Debug)]
pub enum HttpError {
    ReadFailed,
    WriteFailed,
    ConnectionClosed,
    HeadersTooLarge,
    InvalidUtf8,
    MalformedRequest,
    SerializeFailed,
    DeserializeFailed,
}

// =========================================================================
// Request parsing
// =========================================================================

/// Read a complete HTTP/1.1 request (headers + body) from `socket`.
pub async fn read_http_request<S: Read + Unpin>(
    socket: &mut S,
) -> Result<HttpRequest, HttpError> {
    let mut buf = Vec::new();
    let mut byte = [0u8; 1];

    // Read until the end-of-headers marker (\r\n\r\n).
    loop {
        let n =
            socket.read(&mut byte).await.map_err(|_| HttpError::ReadFailed)?;
        if n == 0 {
            return Err(HttpError::ConnectionClosed);
        }
        buf.push(byte[0]);
        if buf.len() >= 4 && buf[buf.len() - 4..] == *b"\r\n\r\n" {
            break;
        }
        if buf.len() > MAX_HEADER_BYTES {
            return Err(HttpError::HeadersTooLarge);
        }
    }

    let header_str =
        core::str::from_utf8(&buf).map_err(|_| HttpError::InvalidUtf8)?;

    // Parse request line: METHOD PATH HTTP/1.x
    let request_line =
        header_str.lines().next().ok_or(HttpError::MalformedRequest)?;
    let mut parts = request_line.split_whitespace();
    let method: String =
        parts.next().ok_or(HttpError::MalformedRequest)?.into();
    let path: String = parts.next().ok_or(HttpError::MalformedRequest)?.into();

    // Find Content-Length header (case-insensitive).
    let mut content_length: usize = 0;
    for line in header_str.lines().skip(1) {
        if line.len() > 16 && line[..15].eq_ignore_ascii_case("content-length:")
        {
            content_length = line[15..].trim().parse().unwrap_or(0);
        }
    }

    // Read body.
    let mut body = Vec::new();
    if content_length > 0 {
        body.resize(content_length, 0);
        let mut read = 0;
        while read < content_length {
            let n = socket
                .read(&mut body[read..])
                .await
                .map_err(|_| HttpError::ReadFailed)?;
            if n == 0 {
                break;
            }
            read += n;
        }
        body.truncate(read);
    }

    Ok(HttpRequest { method, path, body })
}

// =========================================================================
// Response writing
// =========================================================================

/// Write an HTTP response with a JSON body.
pub async fn write_json<S: Write + Unpin>(
    socket: &mut S,
    status: u16,
    body: &serde_json::Value,
) -> Result<(), HttpError> {
    let body_bytes =
        serde_json::to_vec(body).map_err(|_| HttpError::SerializeFailed)?;
    let status_text = status_reason(status);
    let header = format!(
        "HTTP/1.1 {} {}\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\r\n",
        status,
        status_text,
        body_bytes.len()
    );

    socket
        .write_all(header.as_bytes())
        .await
        .map_err(|_| HttpError::WriteFailed)?;
    socket.write_all(&body_bytes).await.map_err(|_| HttpError::WriteFailed)?;
    Ok(())
}

/// Write an HTTP 202 Accepted response with a simple JSON body.
pub async fn write_accepted<S: Write + Unpin>(
    socket: &mut S,
) -> Result<(), HttpError> {
    write_json(socket, 202, &serde_json::json!({"status": "accepted"})).await
}

/// Write an error response derived from a HAL `ErrorInfo`.
pub async fn write_hal_error<S: Write + Unpin>(
    socket: &mut S,
    err: &ErrorInfo,
) -> Result<(), HttpError> {
    write_json(
        socket,
        500,
        &serde_json::json!({
            "error": {
                "code": err.code,
                "message": err.message,
                "hint": err.hint,
                "recoverable": err.recoverable,
            }
        }),
    )
    .await
}

// =========================================================================
// Helpers
// =========================================================================

/// Deserialize a JSON body from an `HttpRequest`.
pub fn parse_body<T: serde::de::DeserializeOwned>(
    request: &HttpRequest,
) -> Result<T, HttpError> {
    serde_json::from_slice(&request.body)
        .map_err(|_| HttpError::DeserializeFailed)
}

/// Map HTTP status codes to reason phrases.
fn status_reason(code: u16) -> &'static str {
    match code {
        200 => "OK",
        202 => "Accepted",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        _ => "OK",
    }
}
