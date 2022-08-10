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
use opencv::{core, imgproc, objdetect, prelude::*, types};
use zenoh_flow::async_std::sync::{Arc, Mutex};
use zenoh_flow::{
    zenoh_flow_derive::ZFState, zf_spin_lock, AsyncIteration, Configuration, Context, Data, Inputs,
    Message, Node, Operator, Outputs, Streams, ZFError, ZFResult,
};

#[derive(Debug)]
struct FaceDetection;

static INPUT: &str = "Frame";
static OUTPUT: &str = "Frame";

#[derive(ZFState, Clone)]
struct FDState {
    pub face: Arc<Mutex<objdetect::CascadeClassifier>>,
    pub encode_options: Arc<Mutex<opencv::types::VectorOfi32>>,
}

// because of opencv
impl std::fmt::Debug for FDState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "FDState:...",)
    }
}

impl FDState {
    fn new(configuration: &Option<Configuration>) -> Self {
        let default_neural_network = &"haarcascades/haarcascade_frontalface_alt.xml".to_owned();
        let neural_network = if let Some(configuration) = configuration {
            configuration["neural-network"]
                .as_str()
                .unwrap_or(default_neural_network)
        } else {
            default_neural_network
        };

        let xml = core::find_file(neural_network, true, false).unwrap();
        let face = objdetect::CascadeClassifier::new(&xml).unwrap();
        let encode_options = opencv::types::VectorOfi32::new();

        Self {
            face: Arc::new(Mutex::new(face)),
            encode_options: Arc::new(Mutex::new(encode_options)),
        }
    }

    pub fn infer(&self, frame: Vec<u8>) -> Vec<u8> {
        let mut face = zf_spin_lock!(self.face);
        let encode_options = zf_spin_lock!(self.encode_options);

        // Decode Image
        let mut frame = opencv::imgcodecs::imdecode(
            &opencv::types::VectorOfu8::from_iter(frame),
            opencv::imgcodecs::IMREAD_COLOR,
        )
        .unwrap();

        let mut gray = Mat::default();
        imgproc::cvt_color(&frame, &mut gray, imgproc::COLOR_BGR2GRAY, 0).unwrap();
        let mut reduced = Mat::default();
        imgproc::resize(
            &gray,
            &mut reduced,
            core::Size {
                width: 0,
                height: 0,
            },
            0.25f64,
            0.25f64,
            imgproc::INTER_LINEAR,
        )
        .unwrap();
        let mut faces = types::VectorOfRect::new();
        face.detect_multi_scale(
            &reduced,
            &mut faces,
            1.1,
            2,
            objdetect::CASCADE_SCALE_IMAGE,
            core::Size {
                width: 30,
                height: 30,
            },
            core::Size {
                width: 0,
                height: 0,
            },
        )
        .unwrap();
        for face in faces {
            let scaled_face = core::Rect {
                x: face.x * 4,
                y: face.y * 4,
                width: face.width * 4,
                height: face.height * 4,
            };
            imgproc::rectangle(
                &mut frame,
                scaled_face,
                core::Scalar::new(0f64, 255f64, -1f64, -1f64),
                10,
                1,
                0,
            )
            .unwrap();
        }

        let mut buf = opencv::types::VectorOfu8::new();
        opencv::imgcodecs::imencode(".jpg", &frame, &mut buf, &encode_options).unwrap();
        buf.into()
    }
}

#[async_trait]
impl Operator for FaceDetection {
    async fn setup(
        &self,
        _context: &mut Context,
        configuration: &Option<Configuration>,
        mut inputs: Inputs,
        mut outputs: Outputs,
    ) -> ZFResult<Option<Arc<dyn AsyncIteration>>> {
        let state = FDState::new(configuration);

        let input_frame = inputs.take(INPUT).unwrap();
        let output_frame = outputs.take(OUTPUT).unwrap();

        Ok(Some(Arc::new(async move || {
            let data = match input_frame.recv_async().await.unwrap() {
                Message::Data(mut msg) => Ok(msg.get_inner_data().try_as_bytes()?.as_ref().clone()),
                _ => Err(ZFError::InvalidData("No data".to_string())),
            }?;

            let buf = state.infer(data);

            output_frame.send_async(Data::from_bytes(buf), None).await
        })))
    }
}

#[async_trait]
impl Node for FaceDetection {
    async fn finalize(&self) -> ZFResult<()> {
        Ok(())
    }
}

// Also generated by macro
zenoh_flow::export_operator!(register);

fn register() -> ZFResult<Arc<dyn Operator>> {
    Ok(Arc::new(FaceDetection) as Arc<dyn Operator>)
}
