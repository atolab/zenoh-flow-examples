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

use async_std::sync::{Arc, Mutex};
use async_trait::async_trait;
use opencv::{core, prelude::*, videoio};
use zenoh_flow::{
    types::ZFResult, zenoh_flow_derive::ZFState, zf_spin_lock, AsyncIteration, Configuration,
    Context, Data, Node, Outputs, Source, Streams, ZFError,
};

#[derive(Debug)]
struct CameraSource;

#[derive(ZFState, Clone)]
struct CameraState {
    pub camera: Arc<Mutex<videoio::VideoCapture>>,
    pub encode_options: Arc<Mutex<opencv::types::VectorOfi32>>,
    pub resolution: (i32, i32),
    pub delay: u64,
}

// because of opencv
impl std::fmt::Debug for CameraState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "CameraState: resolution:{:?} delay:{:?}",
            self.resolution, self.delay
        )
    }
}

impl CameraState {
    fn new(configuration: &Option<Configuration>) -> Self {
        let (camera, resolution, delay) = match configuration {
            Some(configuration) => {
                let camera = match configuration["camera"].as_str() {
                    Some(configured_camera) => {
                        videoio::VideoCapture::from_file(configured_camera, videoio::CAP_ANY)
                            .unwrap()
                    }
                    None => videoio::VideoCapture::new(0, videoio::CAP_ANY).unwrap(),
                };

                let configured_resolution = match configuration["resolution"].as_str() {
                    Some(res) => {
                        let v = res.split('x').collect::<Vec<&str>>();
                        (v[0].parse::<i32>().unwrap(), v[1].parse::<i32>().unwrap())
                    }
                    None => (800, 600),
                };

                let delay = match configuration["fps"].as_f64() {
                    Some(fps) => {
                        let delay: f64 = 1f64 / fps;
                        (delay * 1000f64) as u64
                    }
                    None => 40,
                };

                (camera, configured_resolution, delay)
            }

            None => (
                (videoio::VideoCapture::new(0, videoio::CAP_ANY).unwrap()),
                (800, 600),
                40,
            ),
        };

        let opened = videoio::VideoCapture::is_opened(&camera).unwrap();
        if !opened {
            panic!("Unable to open default camera!");
        }
        let encode_options = opencv::types::VectorOfi32::new();

        Self {
            camera: Arc::new(Mutex::new(camera)),
            encode_options: Arc::new(Mutex::new(encode_options)),
            resolution,
            delay,
        }
    }

    pub fn get_frame(&self) -> Vec<u8> {
        let mut cam = zf_spin_lock!(self.camera);
        let encode_options = zf_spin_lock!(self.encode_options);

        let mut frame = core::Mat::default();
        cam.read(&mut frame).unwrap();

        let mut reduced = Mat::default();
        opencv::imgproc::resize(
            &frame,
            &mut reduced,
            opencv::core::Size::new(self.resolution.0, self.resolution.0),
            0.0,
            0.0,
            opencv::imgproc::INTER_LINEAR,
        )
        .unwrap();

        let mut buf = opencv::types::VectorOfu8::new();
        opencv::imgcodecs::imencode(".jpg", &reduced, &mut buf, &encode_options).unwrap();

        buf.into()
    }
}

#[async_trait]
impl Node for CameraSource {
    async fn finalize(&self) -> ZFResult<()> {
        Ok(())
    }
}

#[async_trait]
impl Source for CameraSource {
    async fn setup(
        &self,
        _context: &mut Context,
        configuration: &Option<Configuration>,
        mut outputs: Outputs,
    ) -> ZFResult<Option<Arc<dyn AsyncIteration>>> {
        let state = CameraState::new(configuration);

        let output = outputs.take("Frame").ok_or(ZFError::NotFound)?;

        Ok(Some(Arc::new(async move || {
            let buf = state.get_frame();
            output
                .send_async(Data::from_bytes(buf), None)
                .await
                .unwrap();
            async_std::task::sleep(std::time::Duration::from_millis(state.delay)).await;
            Ok(())
        })))
    }
}

// Also generated by macro
zenoh_flow::export_source!(register);

fn register() -> ZFResult<Arc<dyn Source>> {
    Ok(Arc::new(CameraSource) as Arc<dyn Source>)
}
