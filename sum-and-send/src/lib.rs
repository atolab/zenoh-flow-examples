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
use zenoh_flow::zenoh_flow_derive::ZFState;
use zenoh_flow::{AsyncIteration, Configuration, Inputs, Message, Outputs};
use zenoh_flow::{Data, Node, Operator, ZFError, ZFResult};
use zenoh_flow_example_types::ZFUsize;

#[derive(Debug)]
struct SumAndSend;

#[derive(Debug, Clone, ZFState)]
struct SumAndSendState {
    pub x: ZFUsize,
}

static INPUT: &str = "Number";
static OUTPUT: &str = "Sum";
#[async_trait]
impl Operator for SumAndSend {
    async fn setup(
        &self,
        _configuration: &Option<Configuration>,
        mut inputs: Inputs,
        mut outputs: Outputs,
    ) -> ZFResult<Arc<dyn AsyncIteration>> {
        let mut state = SumAndSendState { x: ZFUsize(0) };

        let input_value = inputs.remove(INPUT).unwrap();
        let output_value = outputs.remove(OUTPUT).unwrap();

        Ok(Arc::new(async move || {
            let data = match input_value.recv().await.unwrap() {
                Message::Data(mut msg) => Ok(msg.get_inner_data().try_get::<ZFUsize>()?.clone()),
                _ => Err(ZFError::InvalidData("No data".to_string())),
            }?;

            let res = ZFUsize(state.x.0 + data.0);
            state.x = res.clone();

            output_value.send(Data::from(res), None).await
        }))
    }
}

#[async_trait]
impl Node for SumAndSend {
    async fn finalize(&self) -> ZFResult<()> {
        Ok(())
    }
}

// Also generated by macro
zenoh_flow::export_operator!(register);

fn register() -> ZFResult<Arc<dyn Operator>> {
    Ok(Arc::new(SumAndSend) as Arc<dyn Operator>)
}
