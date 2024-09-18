use safe_drive::{context::Context, error::DynError};
use vl53l1x;

use std::time::Duration;

fn calc_md_power(distance: u16) -> i16 {
    // distanceに応じてモーターのPWMを計算
    // 戻り値は-1000~1000の間に収まるように
    return 1000;
}

#[tokio::main]
async fn main() -> Result<(), DynError> {
    let mut vl = vl53l1x::Vl53l1x::new(7, None)?;
    vl.soft_reset()?;
    vl.init()?;
    vl.start_ranging(vl53l1x::DistanceMode::Short)?;

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
    let mut before_distance = 0;
    selector.add_wall_timer(
        "publisher",
        Duration::from_millis(100),
        Box::new(move || {
            let distance = match vl.read_sample() {
                Ok(t) => t.distance,
                Err(_) => {
                    before_distance
                }
            };
            before_distance = distance;
            msg.power = calc_md_power(distance);
            let _ = publisher.send(&msg);
        })
    );

    loop {
        selector.wait()?;
    }
}
