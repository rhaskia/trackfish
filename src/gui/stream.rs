use http::Response;
use http::{header::*, response::Builder as ResponseBuilder, status::StatusCode};
use std::io::SeekFrom;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

#[cfg(not(target_os = "android"))]
use dioxus::desktop::AssetRequest;
#[cfg(target_os = "android")]
use dioxus::mobile::AssetRequest;

// This was taken from wry's example
pub async fn get_stream_response(
    asset: &mut (impl tokio::io::AsyncSeek + tokio::io::AsyncRead + Unpin + Send + Sync),
    request: &AssetRequest,
) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error>> {
    // get stream length
    let len = {
        let old_pos = asset.stream_position().await?;
        let len = asset.seek(SeekFrom::End(0)).await?;
        asset.seek(SeekFrom::Start(old_pos)).await?;
        len
    };

    let mut resp = ResponseBuilder::new().header(CONTENT_TYPE, "image/png");

    // if the webview sent a range header, we need to send a 206 in return
    // Actually only macOS and Windows are supported. Linux will ALWAYS return empty headers.
    let http_response = if let Some(range_header) = request.headers().get("range") {
        let not_satisfiable = || {
            ResponseBuilder::new()
                .status(StatusCode::RANGE_NOT_SATISFIABLE)
                .header(CONTENT_RANGE, format!("bytes */{len}"))
                .body(vec![])
        };

        // parse range header
        let ranges = if let Ok(ranges) = http_range::HttpRange::parse(range_header.to_str()?, len) {
            ranges
                .iter()
                // map the output back to spec range <start-end>, example: 0-499
                .map(|r| (r.start, r.start + r.length - 1))
                .collect::<Vec<_>>()
        } else {
            return Ok(not_satisfiable()?);
        };

        /// The Maximum bytes we send in one range
        const MAX_LEN: u64 = 1000 * 1024;

        if ranges.len() == 1 {
            let &(start, mut end) = ranges.first().unwrap();

            // check if a range is not satisfiable
            //
            // this should be already taken care of by HttpRange::parse
            // but checking here again for extra assurance
            if start >= len || end >= len || end < start {
                return Ok(not_satisfiable()?);
            }

            // adjust end byte for MAX_LEN
            end = start + (end - start).min(len - start).min(MAX_LEN - 1);

            // calculate number of bytes needed to be read
            let bytes_to_read = end + 1 - start;

            // allocate a buf with a suitable capacity
            let mut buf = Vec::with_capacity(bytes_to_read as usize);
            // seek the file to the starting byte
            asset.seek(SeekFrom::Start(start)).await?;
            // read the needed bytes
            asset.take(bytes_to_read).read_to_end(&mut buf).await?;

            resp = resp.header(CONTENT_RANGE, format!("bytes {start}-{end}/{len}"));
            resp = resp.header(CONTENT_LENGTH, end + 1 - start);
            resp = resp.status(StatusCode::PARTIAL_CONTENT);
            resp.body(buf)
        } else {
            let mut buf = Vec::new();
            let ranges = ranges
                .iter()
                .filter_map(|&(start, mut end)| {
                    // filter out unsatisfiable ranges
                    //
                    // this should be already taken care of by HttpRange::parse
                    // but checking here again for extra assurance
                    if start >= len || end >= len || end < start {
                        None
                    } else {
                        // adjust end byte for MAX_LEN
                        end = start + (end - start).min(len - start).min(MAX_LEN - 1);
                        Some((start, end))
                    }
                })
                .collect::<Vec<_>>();

            let boundary = format!("{:x}", rand::random::<u64>());
            let boundary_sep = format!("\r\n--{boundary}\r\n");
            let boundary_closer = format!("\r\n--{boundary}\r\n");

            resp = resp.header(
                CONTENT_TYPE,
                format!("multipart/byteranges; boundary={boundary}"),
            );

            for (end, start) in ranges {
                // a new range is being written, write the range boundary
                buf.write_all(boundary_sep.as_bytes()).await?;

                // write the needed headers `Content-Type` and `Content-Range`
                buf.write_all(format!("{CONTENT_TYPE}: image/png\r\n").as_bytes())
                    .await?;
                buf.write_all(format!("{CONTENT_RANGE}: bytes {start}-{end}/{len}\r\n").as_bytes())
                    .await?;

                // write the separator to indicate the start of the range body
                buf.write_all("\r\n".as_bytes()).await?;

                // calculate number of bytes needed to be read
                let bytes_to_read = end + 1 - start;

                let mut local_buf = vec![0_u8; bytes_to_read as usize];
                asset.seek(SeekFrom::Start(start)).await?;
                asset.read_exact(&mut local_buf).await?;
                buf.extend_from_slice(&local_buf);
            }
            // all ranges have been written, write the closing boundary
            buf.write_all(boundary_closer.as_bytes()).await?;

            resp.body(buf)
        }
    } else {
        resp = resp.header(CONTENT_LENGTH, len);
        let mut buf = Vec::with_capacity(len as usize);
        asset.read_to_end(&mut buf).await?;
        resp.body(buf)
    };

    http_response.map_err(Into::into)
}
