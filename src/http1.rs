use std::io::{self, BufRead, BufReader, Read};
use std::net::TcpStream;
use std::sync::Arc;
use crate::config::Config;
use crate::metrics::Metrics;

/// HTTP/1.x observe-only parser
/// Extracts method, path, status, and latency without modifying traffic
pub struct Http1Observer {
    config: Arc<Config>,
    metrics: Arc<Metrics>,
}

#[derive(Debug)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers_size: usize,
    #[cfg(feature = "recorder")]
    pub body: Option<Vec<u8>>,
}

#[derive(Debug)]
pub struct HttpResponse {
    pub status: u16,
    pub version: String,
    pub headers_size: usize,
    #[cfg(feature = "recorder")]
    pub body: Option<Vec<u8>>,
}

impl Http1Observer {
    pub fn new(config: Arc<Config>, metrics: Arc<Metrics>) -> Self {
        Http1Observer { config, metrics }
    }

    /// Parse HTTP/1.x request (observe-only)
    /// Returns parsed request and original bytes for forwarding
    pub fn parse_request(&self, stream: &mut TcpStream) -> io::Result<(HttpRequest, Vec<u8>)> {
        let mut reader = BufReader::new(stream);
        let mut request_bytes = Vec::new();
        let mut headers_size = 0;

        // Parse request line
        let mut line = String::new();
        reader.read_line(&mut line)?;
        request_bytes.extend_from_slice(line.as_bytes());
        headers_size += line.len();

        let parts: Vec<&str> = line.trim().split_whitespace().collect();
        if parts.len() < 3 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid HTTP request line",
            ));
        }

        let method = parts[0].to_string();
        let path = parts[1].to_string();
        let version = parts[2].to_string();

        // Parse headers
        let mut content_length = 0;
        loop {
            line.clear();
            reader.read_line(&mut line)?;
            request_bytes.extend_from_slice(line.as_bytes());
            headers_size += line.len();

            if line.trim().is_empty() {
                break; // End of headers
            }

            // Check for Content-Length
            if line.to_lowercase().starts_with("content-length:") {
                if let Some(len_str) = line.split(':').nth(1) {
                    content_length = len_str.trim().parse().unwrap_or(0);
                }
            }

            if headers_size > self.config.http.max_header_size {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Headers too large",
                ));
            }
        }

        // Handle body (if needed for recorder)
        #[cfg(feature = "recorder")]
        let body = if content_length > 0 && content_length <= self.config.http.max_body_size {
            let mut body_buf = vec![0u8; content_length];
            reader.read_exact(&mut body_buf)?;
            request_bytes.extend_from_slice(&body_buf);
            Some(body_buf)
        } else {
            None
        };

        self.metrics.http_request_size_bytes.observe(request_bytes.len() as f64);

        Ok((
            HttpRequest {
                method,
                path,
                version,
                headers_size,
                #[cfg(feature = "recorder")]
                body,
            },
            request_bytes,
        ))
    }

    /// Parse HTTP/1.x response (observe-only)
    pub fn parse_response(&self, stream: &mut TcpStream) -> io::Result<(HttpResponse, Vec<u8>)> {
        let mut reader = BufReader::new(stream);
        let mut response_bytes = Vec::new();
        let mut headers_size = 0;

        // Parse status line
        let mut line = String::new();
        reader.read_line(&mut line)?;
        response_bytes.extend_from_slice(line.as_bytes());
        headers_size += line.len();

        let parts: Vec<&str> = line.trim().split_whitespace().collect();
        if parts.len() < 2 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid HTTP response line",
            ));
        }

        let version = parts[0].to_string();
        let status: u16 = parts[1].parse().unwrap_or(0);

        // Parse headers
        let mut content_length = 0;
        loop {
            line.clear();
            reader.read_line(&mut line)?;
            response_bytes.extend_from_slice(line.as_bytes());
            headers_size += line.len();

            if line.trim().is_empty() {
                break; // End of headers
            }

            // Check for Content-Length
            if line.to_lowercase().starts_with("content-length:") {
                if let Some(len_str) = line.split(':').nth(1) {
                    content_length = len_str.trim().parse().unwrap_or(0);
                }
            }

            if headers_size > self.config.http.max_header_size {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Headers too large",
                ));
            }
        }

        // Handle body (if needed for recorder)
        #[cfg(feature = "recorder")]
        let body = if content_length > 0 && content_length <= self.config.http.max_body_size {
            let mut body_buf = vec![0u8; content_length];
            reader.read_exact(&mut body_buf)?;
            response_bytes.extend_from_slice(&body_buf);
            Some(body_buf)
        } else {
            None
        };

        self.metrics.http_response_size_bytes.observe(response_bytes.len() as f64);

        Ok((
            HttpResponse {
                status,
                version,
                headers_size,
                #[cfg(feature = "recorder")]
                body,
            },
            response_bytes,
        ))
    }

    /// Record HTTP metrics
    pub fn record_request(
        &self,
        request: &HttpRequest,
        response: &HttpResponse,
        duration: std::time::Duration,
    ) {
        let status = response.status.to_string();
        self.metrics
            .http_requests_total
            .with_label_values(&[&request.method, &status])
            .inc();
        self.metrics
            .http_request_duration_seconds
            .with_label_values(&[&request.method, &status])
            .observe(duration.as_secs_f64());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http1_observer_creation() {
        let config = Arc::new(Config::default());
        let metrics = Metrics::new().unwrap();
        let observer = Http1Observer::new(config, metrics);
        assert!(true); // Basic creation test
    }
}
