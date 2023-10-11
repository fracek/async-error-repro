use core::pin::pin;
use std::{fmt::Debug, sync::Arc, time::Duration};

use futures::{Future, Stream, StreamExt, TryStreamExt};
use k8s_openapi::api::core::v1::Pod;
use kube::{
    runtime::{
        controller::{self, Action},
        reflector::ObjectRef,
        watcher, Controller,
    },
    Api, Client, Resource,
};

#[derive(thiserror::Error, Debug)]
enum MyError {}

struct Context {
    client: Client,
}

type ReconcileItem<K> = Result<(ObjectRef<K>, Action), controller::Error<MyError, watcher::Error>>;

async fn reconcile(_pod: Arc<Pod>, _ctx: Arc<Context>) -> Result<Action, MyError> {
    Ok(Action::requeue(Duration::from_secs(10)))
}

fn error_policy(_pod: Arc<Pod>, _err: &MyError, _ctx: Arc<Context>) -> Action {
    Action::requeue(Duration::from_secs(10))
}

async fn create_controller(
    client: Client,
) -> Result<impl Stream<Item = ReconcileItem<Pod>>, MyError> {
    let pods = Api::<Pod>::all(client.clone());
    let context = Context { client };

    let controller = Controller::new(pods, watcher::Config::default()).run(
        reconcile,
        error_policy,
        context.into(),
    );

    Ok(controller)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::try_default().await?;
    // let mut controller = create_controller(client).await?;
    let mut controller = pin!(create_controller(client).await?);
    while let Some(_item) = controller.try_next().await? {
    }
    Ok(())
}
