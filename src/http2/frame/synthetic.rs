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

// std
use std::mem;

// osmium
use http2::stream::StreamId;
use http2::frame::{self as framing, CompressibleHttpFrame};

#[derive(Debug)]
pub struct HeaderBlockSyntheticFrame {
    header_frame: framing::headers::HeadersFrameCompressModel,
    continuation_frames: Vec<framing::continuation::ContinuationFrameCompressModel>
}

impl HeaderBlockSyntheticFrame {
    pub fn new(header_frame: framing::headers::HeadersFrameCompressModel) -> Self {
        HeaderBlockSyntheticFrame {
            header_frame: header_frame,
            continuation_frames: Vec::new()
        }
    }

    pub fn push_continuation(&mut self, continuation_frame: framing::continuation::ContinuationFrameCompressModel) {
        self.continuation_frames.push(continuation_frame);
    }
}

impl CompressibleHttpFrame for HeaderBlockSyntheticFrame {
    /// Yields the size of the HEADERS frame, ignoring any continuation frames
    fn get_length(&self) -> i32 {
        self.header_frame.get_length()
    }

    /// Yields HEADERS frame type, ignoring any continuation frames
    fn get_frame_type(&self) -> framing::FrameType {
        framing::FrameType::Headers
    }

    // Yields the size of the HEADERS frame, ignoring any continuation frames
    fn get_flags(&self) -> u8 {
        self.header_frame.get_flags()
    }

    /// Delegates to the implementations of this trait for HEADERS and CONTINUATION frames
    fn get_payload(self: Box<Self>) -> Vec<u8> {
        Box::new(self.header_frame).get_payload()
    }

    /// Override the default compression, compressing the header and any continuation frames at the same time
    fn compress_frame(mut self: Box<Self>, stream_id: StreamId) -> Vec<u8> {
        let mut temp_header_frame = framing::headers::HeadersFrameCompressModel::new(false, false);
        mem::swap(&mut self.header_frame, &mut temp_header_frame);
        let mut result = Box::new(temp_header_frame).compress_frame(stream_id);

        for cont in self.continuation_frames.into_iter() {
            result.extend(Box::new(cont).compress_frame(stream_id));
        }

        result
    }
}

