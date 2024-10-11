use autd3::{
    link::Audit,
    prelude::{Vector3, AUTD3},
    Controller,
};

mod datagram;
mod link;

#[tokio::test]
async fn initial_msg_id() {
    assert!(Controller::builder([AUTD3::new(Vector3::zeros())])
        .open(Audit::builder().with_initial_msg_id(Some(0x01)))
        .await
        .is_ok());
}
