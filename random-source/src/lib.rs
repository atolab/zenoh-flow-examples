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
use zenoh_flow::{types::ZFResult, Data, Node, Source};
use zenoh_flow::{AsyncIteration, Configuration, Outputs};
use zenoh_flow_example_types::ZFUsize;

#[derive(Debug)]
struct ExampleRandomSource;

#[async_trait]
impl Node for ExampleRandomSource {
    async fn finalize(&self) -> ZFResult<()> {
        Ok(())
    }
}

#[async_trait]
impl Source for ExampleRandomSource {
    async fn setup(
        &self,
        _configuration: &Option<Configuration>,
        mut outputs: Outputs,
    ) -> ZFResult<Arc<dyn AsyncIteration>> {
        let output = outputs.remove("Random").unwrap();

        Ok(Arc::new(async move || {
            zenoh_flow::async_std::task::sleep(std::time::Duration::from_secs(1)).await;
            output
                .send_async(Data::from(ZFUsize(rand::random::<usize>())), None)
                .await
        }))
    }
}

// Also generated by macro
zenoh_flow::export_source!(register);

fn register() -> ZFResult<Arc<dyn Source>> {
    Ok(Arc::new(ExampleRandomSource) as Arc<dyn Source>)
}
