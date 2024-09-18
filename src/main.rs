use safe_drive::{context::Context, error::DynError};
use drobo_interfaces::msg::MdLibMsg;
use vl53l1x;

use std::time::Duration;

fn calc_md_power(distance: u16) -> u16 {
    // distanceに応じてモーターのPWMを計算
    // 戻り値は-1000~1000の間に収まるように
}

#[tokio::main]
async fn main() -> Result<(), DynError> {
    let mut vl = vl53l1x::Vl53l1x::new(1, None)?;
    vl.soft_reset()?;
    vl.init()?;
    vl.start_ranging(vl53l1x::DistanceMode::Mid)?;

    let ctx = Context::new()?;
    let node = ctx.create_node("pole_detector", None, Default::default())?;
    let publisher =
        node.create_publisher::<drobo_interfaces::msg::MdLibMsg>("md_driver_topic", None)?;

    let mut msg = drobo_interfaces::msg::MdLibMsg::new().unwrap();
    msg.address = 0x05;
    msg.mode = 2;
    msg.phase = false;
    msg.timeout = 1000;

    let mut selector = ctx.create_selector()?;
    selector.add_wall_timer(
        "publisher",
        Duration::from_millis(100),
        Box::new(move || {
            let distance = vl.read_sample()?.distance;
            msg.power = calc_md_power(distance);
            publisher.send(&msg);
        })
    )

    loop {
        selector.wait()?;
    }
}
