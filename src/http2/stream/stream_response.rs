// Copyright 2017 ThetaSinner
//
// This file is part of Osmium.

// Osmium is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Osmium is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Osmium. If not, see <http://www.gnu.org/licenses/>.

// osmium
use http2::header;
use http2::frame as framing;
use http2::hpack::context as hpack_context;
use http2::hpack::pack as hpack_pack;

#[derive(Debug)]
pub struct StreamResponse {
    pub informational_headers: Vec<header::Headers>,
    pub headers: header::Headers,
    pub payload: Option<Vec<u8>>,
    pub trailer_headers: Option<header::Headers>
}

impl StreamResponse {
    pub fn to_frames(self, hpack_send_context: &mut hpack_context::SendContext) -> Vec<Box<framing::CompressibleHttpFrame>>
    {
        trace!("Starting to convert stream response to frames [{:?}]", self);

        let mut frames: Vec<Box<framing::CompressibleHttpFrame>> = Vec::new();

        for informational_header in &self.informational_headers {
            frames.extend(
                StreamResponse::headers_to_frames(informational_header, hpack_send_context, false)
            );
        }

        let headers = StreamResponse::headers_to_frames(&self.headers, hpack_send_context, self.payload.is_none() && self.trailer_headers.is_none());
        frames.extend(headers);

        if self.payload.is_some() {
            let mut data_frame = framing::data::DataFrameCompressModel::new(false);
            data_frame.set_payload(self.payload.unwrap());
            if self.trailer_headers.is_none() {
                data_frame.set_end_stream();
            }
            frames.push(Box::new(data_frame));
        }

        if self.trailer_headers.is_some() {
            let trailer_headers_frame = StreamResponse::headers_to_frames(&self.trailer_headers.unwrap(), hpack_send_context, true);
            frames.extend(trailer_headers_frame);
        }

        trace!("Converted to frames [{:?}]", frames);

        frames
    }

    fn headers_to_frames(headers: &header::Headers, hpack_send_context: &mut hpack_context::SendContext, end_stream: bool) -> Vec<Box<framing::CompressibleHttpFrame>>
    {
        let mut temp_frames: Vec<Box<framing::CompressibleHttpFrame>> = Vec::new();

        let packed = hpack_pack::pack(&headers, hpack_send_context, true);
        let num_chunks = ((packed.len() as f32) / 150f32).ceil() as i32;
        let mut chunk_count = 1;
        let mut chunks = packed.chunks(150);

        if let Some(first_chunk) = chunks.next() {
            let mut headers_frame = framing::headers::HeadersFrameCompressModel::new(false, false);
            if end_stream {
                headers_frame.set_end_stream();
            }
            if num_chunks == 1 {
                headers_frame.set_end_headers();
            }
            headers_frame.set_header_block_fragment(first_chunk.to_vec());
            temp_frames.push(Box::new(headers_frame));
        }
        else {
            panic!("empty headers block");
        }
        
        while let Some(chunk) = chunks.next() {
            let mut headers_frame = framing::headers::HeadersFrameCompressModel::new(false, false);
            headers_frame.set_header_block_fragment(chunk.to_vec());

            chunk_count += 1;
            if chunk_count == num_chunks {
                headers_frame.set_end_headers();
            }

            temp_frames.push(Box::new(headers_frame));
        }

        temp_frames
    }
}
