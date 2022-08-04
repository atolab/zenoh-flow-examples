//
// Copyright (c) 2022 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//
#![feature(async_closure)]

use async_trait::async_trait;
use zenoh_flow::async_std::sync::Arc;
use zenoh_flow::{types::ZFResult, zenoh_flow_derive::ZFState, Node, Sink, Streams};
use zenoh_flow::{AsyncIteration, Configuration, Inputs, Message, ZFError};

use opencv::{highgui, prelude::*};

#[derive(Debug)]
struct VideoSink;

#[derive(ZFState, Clone, Debug)]
struct VideoState {
    pub window_name: String,
}

impl VideoState {
    pub fn new() -> Self {
        let window_name = &"Video-Sink".to_string();
        highgui::named_window(window_name, 1).unwrap();
        Self {
            window_name: window_name.to_string(),
        }
    }

    pub fn show(&self, frame: Vec<u8>) {
        let decoded = opencv::imgcodecs::imdecode(
            &opencv::types::VectorOfu8::from_iter(frame),
            opencv::imgcodecs::IMREAD_COLOR,
        )
        .unwrap();

        if decoded.size().unwrap().width > 0 {
            highgui::imshow(&self.window_name, &decoded).unwrap();
        }

        highgui::wait_key(10).unwrap();
    }
}

#[async_trait]
impl Node for VideoSink {
    async fn finalize(&self) -> ZFResult<()> {
        // let state = state.try_get::<VideoState>()?;
        // highgui::destroy_window(&state.window_name).unwrap();
        Ok(())
    }
}

#[async_trait]
impl Sink for VideoSink {
    async fn setup(
        &self,
        _configuration: &Option<Configuration>,
        mut inputs: Inputs,
    ) -> ZFResult<Arc<dyn AsyncIteration>> {
        let state = VideoState::new();
        let input = inputs.take("Frame").unwrap();

        Ok(Arc::new(async move || {
            let frame = match input.recv_async().await.unwrap() {
                Message::Data(mut msg) => Ok(msg.get_inner_data().try_as_bytes()?.as_ref().clone()),
                _ => Err(ZFError::InvalidData("No data".to_string())),
            }?;

            state.show(frame);
            Ok(())
        }))
    }
}

// Also generated by macro
zenoh_flow::export_sink!(register);

fn register() -> ZFResult<Arc<dyn Sink>> {
    Ok(Arc::new(VideoSink) as Arc<dyn Sink>)
}
