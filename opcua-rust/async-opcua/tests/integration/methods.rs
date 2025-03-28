use std::{
    sync::{atomic::AtomicU64, Arc},
    time::Duration,
};

use crate::utils::ChannelNotifications;

use super::utils::setup;
use opcua::{
    server::address_space::MethodBuilder,
    types::{
        AttributeId, CallMethodRequest, DataTypeId, NodeId, ObjectId, StatusCode, Variant,
        VariantTypeId,
    },
};
use opcua_types::{
    MonitoredItemCreateRequest, MonitoringParameters, ReadValueId, TimestampsToReturn, VariableId,
    VariantScalarTypeId,
};

#[tokio::test]
async fn call_trivial() {
    let (_tester, nm, session) = setup().await;
    let called = Arc::new(AtomicU64::new(0));

    let id = nm.inner().next_node_id();
    let input_id = nm.inner().next_node_id();
    let output_id = nm.inner().next_node_id();
    {
        let mut sp = nm.address_space().write();
        MethodBuilder::new(&id, "TestMethod1", "TestMethod1")
            .executable(true)
            .user_executable(true)
            .component_of(ObjectId::ObjectsFolder)
            .input_args(&mut *sp, &input_id, &[])
            .output_args(&mut *sp, &output_id, &[])
            .insert(&mut *sp);
    }

    let called_ref = called.clone();
    nm.inner().add_method_cb(id.clone(), move |_| {
        called_ref.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(vec![])
    });

    let r = session
        .call_one(CallMethodRequest {
            object_id: ObjectId::ObjectsFolder.into(),
            method_id: id.clone(),
            input_arguments: None,
        })
        .await
        .unwrap();
    assert_eq!(r.status_code, StatusCode::Good);
    assert_eq!(1, called.load(std::sync::atomic::Ordering::Relaxed));
}

#[tokio::test]
async fn call_args() {
    let (_tester, nm, session) = setup().await;

    let id = nm.inner().next_node_id();
    let input_id = nm.inner().next_node_id();
    let output_id = nm.inner().next_node_id();
    {
        let mut sp = nm.address_space().write();
        MethodBuilder::new(&id, "MethodAdd", "MethodAdd")
            .executable(true)
            .user_executable(true)
            .component_of(ObjectId::ObjectsFolder)
            .input_args(
                &mut *sp,
                &input_id,
                &[
                    ("Lhs", DataTypeId::Int64).into(),
                    ("Rhs", DataTypeId::Int64).into(),
                ],
            )
            .output_args(
                &mut *sp,
                &output_id,
                &[("Result", DataTypeId::Int64).into()],
            )
            .insert(&mut *sp);
    }

    nm.inner().add_method_cb(id.clone(), |args| {
        let Some(Variant::Int64(lhs)) = args
            .first()
            .map(|a| a.cast(VariantTypeId::Scalar(VariantScalarTypeId::Int64)))
        else {
            return Err(StatusCode::BadInvalidArgument);
        };
        let Some(Variant::Int64(rhs)) = args
            .get(1)
            .map(|a| a.cast(VariantTypeId::Scalar(VariantScalarTypeId::Int64)))
        else {
            return Err(StatusCode::BadInvalidArgument);
        };

        Ok(vec![Variant::Int64(lhs + rhs)])
    });

    let r = session
        .call_one(CallMethodRequest {
            object_id: ObjectId::ObjectsFolder.into(),
            method_id: id.clone(),
            input_arguments: Some(vec![Variant::Int64(3), Variant::Int64(2)]),
        })
        .await
        .unwrap();
    assert_eq!(r.status_code, StatusCode::Good);
    let outputs = r.output_arguments.unwrap().clone();
    assert_eq!(1, outputs.len());
    let Variant::Int64(v) = outputs[0] else {
        panic!("Wrong output type");
    };
    assert_eq!(v, 5);

    // Call with wrong args
    let r = session
        .call_one(CallMethodRequest {
            object_id: ObjectId::ObjectsFolder.into(),
            method_id: id.clone(),
            input_arguments: Some(vec![Variant::String("foo".into()), Variant::Int64(2)]),
        })
        .await
        .unwrap();

    assert_eq!(r.status_code, StatusCode::BadInvalidArgument);
}

#[tokio::test]
async fn call_fail() {
    let (_tester, nm, session) = setup().await;

    let id = nm.inner().next_node_id();
    let input_id = nm.inner().next_node_id();
    let output_id = nm.inner().next_node_id();
    {
        let mut sp = nm.address_space().write();
        MethodBuilder::new(&id, "MethodAdd", "MethodAdd")
            .executable(true)
            .user_executable(false)
            .component_of(ObjectId::ObjectsFolder)
            .input_args(
                &mut *sp,
                &input_id,
                &[
                    ("Lhs", DataTypeId::Int64).into(),
                    ("Rhs", DataTypeId::Int64).into(),
                ],
            )
            .output_args(
                &mut *sp,
                &output_id,
                &[("Result", DataTypeId::Int64).into()],
            )
            .insert(&mut *sp);
    }

    nm.inner().add_method_cb(id.clone(), |args| {
        let Some(Variant::Int64(lhs)) = args.first().map(|a| a.cast(VariantScalarTypeId::Int64))
        else {
            return Err(StatusCode::BadInvalidArgument);
        };
        let Some(Variant::Int64(rhs)) = args.get(1).map(|a| a.cast(VariantScalarTypeId::Int64))
        else {
            return Err(StatusCode::BadInvalidArgument);
        };

        Ok(vec![Variant::Int64(lhs + rhs)])
    });

    // Call method that doesn't exist
    let r = session
        .call_one(CallMethodRequest {
            object_id: ObjectId::ObjectsFolder.into(),
            method_id: NodeId::new(2, 100),
            input_arguments: Some(vec![Variant::Int64(3), Variant::Int64(2)]),
        })
        .await
        .unwrap();
    assert_eq!(r.status_code, StatusCode::BadMethodInvalid);

    // Call on wrong object
    let r = session
        .call_one(CallMethodRequest {
            object_id: ObjectId::Server.into(),
            method_id: id.clone(),
            input_arguments: Some(vec![Variant::Int64(3), Variant::Int64(2)]),
        })
        .await
        .unwrap();
    assert_eq!(r.status_code, StatusCode::BadMethodInvalid);

    // Call without permission
    let r = session
        .call_one(CallMethodRequest {
            object_id: ObjectId::ObjectsFolder.into(),
            method_id: id.clone(),
            input_arguments: Some(vec![Variant::Int64(3), Variant::Int64(2)]),
        })
        .await
        .unwrap();
    assert_eq!(r.status_code, StatusCode::BadUserAccessDenied);

    {
        let mut sp = nm.address_space().write();
        sp.find_mut(&id)
            .unwrap()
            .as_mut_node()
            .set_attribute(AttributeId::UserExecutable, Variant::Boolean(true))
            .unwrap();
    }

    // Call with too many arguments
    let r = session
        .call_one(CallMethodRequest {
            object_id: ObjectId::ObjectsFolder.into(),
            method_id: id.clone(),
            input_arguments: Some(vec![
                Variant::Int64(3),
                Variant::Int64(2),
                Variant::Int64(3),
            ]),
        })
        .await
        .unwrap();
    assert_eq!(r.status_code, StatusCode::BadTooManyArguments);
}

#[tokio::test]
async fn call_limits() {
    let (tester, _nm, session) = setup().await;

    let limit = tester
        .handle
        .info()
        .config
        .limits
        .operational
        .max_nodes_per_method_call;

    // Call none
    let e = session.call(Vec::new()).await.unwrap_err();
    assert_eq!(e, StatusCode::BadNothingToDo);

    // Call too many
    let e = session
        .call(
            (0..(limit + 1))
                .map(|i| CallMethodRequest {
                    object_id: ObjectId::ObjectsFolder.into(),
                    method_id: NodeId::new(2, i as i32),
                    input_arguments: None,
                })
                .collect(),
        )
        .await
        .unwrap_err();
    assert_eq!(e, StatusCode::BadTooManyOperations);
}

#[tokio::test]
async fn call_get_monitored_items() {
    let (_tester, _nm, session) = setup().await;

    let (notifs, _data, _) = ChannelNotifications::new();

    // Create a subscription
    let sub_id = session
        .create_subscription(Duration::from_millis(100), 100, 20, 1000, 0, true, notifs)
        .await
        .unwrap();

    // Create a monitored item on that subscription
    session
        .create_monitored_items(
            sub_id,
            TimestampsToReturn::Both,
            vec![MonitoredItemCreateRequest {
                item_to_monitor: ReadValueId {
                    node_id: VariableId::Server_ServerStatus_State.into(),
                    attribute_id: AttributeId::Value as u32,
                    ..Default::default()
                },
                monitoring_mode: opcua::types::MonitoringMode::Reporting,
                requested_parameters: MonitoringParameters {
                    sampling_interval: 0.0,
                    queue_size: 10,
                    discard_oldest: true,
                    client_handle: 15,
                    ..Default::default()
                },
            }],
        )
        .await
        .unwrap();

    let (ids, handles) = session.call_get_monitored_items(sub_id).await.unwrap();

    assert_eq!(ids.len(), 1);
    assert_eq!(handles.len(), 1);
    assert_eq!(15, handles[0]);
}
