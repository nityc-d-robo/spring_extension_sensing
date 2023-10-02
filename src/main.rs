use gpio_cdev::{Chip, LineRequestFlags};
use safe_drive::{context::Context, error::DynError};
use vl53l1x;

use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), DynError> {
    let mut chip = Chip::new("/dev/gpiochip0")?;
    let front_sensor_pin = 105;
    let mid_sensor_pin = 106;
    let rear_sensor_pin = 43;

    let handle_front_sensor =
        chip.get_line(front_sensor_pin)?
            .request(LineRequestFlags::OUTPUT, 0, "distance_front")?;
    let handle_mid_sensor =
        chip.get_line(mid_sensor_pin)?
            .request(LineRequestFlags::OUTPUT, 0, "distance_mid")?;
    let handle_rear_sensor =
        chip.get_line(rear_sensor_pin)?
            .request(LineRequestFlags::OUTPUT, 0, "distance_rear")?;

    handle_rear_sensor.set_value(1)?;
    let mut vl_rear = vl53l1x::Vl53l1x::new(1, None)?;
    vl_rear.soft_reset()?;
    vl_rear.init()?;
    vl_rear.set_device_address(0x31)?;
    vl_rear.start_ranging(vl53l1x::DistanceMode::Mid)?;

    handle_mid_sensor.set_value(1)?;
    let mut vl_mid = vl53l1x::Vl53l1x::new(1, None)?;
    vl_mid.soft_reset()?;
    vl_mid.init()?;
    vl_mid.set_device_address(0x30)?;
    vl_mid.start_ranging(vl53l1x::DistanceMode::Mid)?;

    handle_front_sensor.set_value(1)?;
    let mut vl_front = vl53l1x::Vl53l1x::new(1, None)?;
    vl_front.soft_reset()?;
    vl_front.init()?;
    vl_front.start_ranging(vl53l1x::DistanceMode::Mid)?;

    let ctx = Context::new()?;
    let node = ctx.create_node("pole_detector", None, Default::default())?;
    let publisher =
        node.create_publisher::<drobo_interfaces::msg::SolenoidStateMsg>("solenoid_order", None)?;
    let mut msg = drobo_interfaces::msg::SolenoidStateMsg::new().unwrap();
    

    loop {
        let s_front = vl_front.read_sample()?.distance;
        if s_front < 1100 {
            // サポート機構を上げる
            let support_address = 0x04;
            msg.axle_position = support_address;
            msg.state = true;
            publisher.send(&msg)?;

            // ここからタイヤの上げ下げ
            msg.axle_position = 0;
            msg.state = true;
            publisher.send(&msg)?;

            while vl_mid.read_sample()?.distance < 1100 {}
            msg.state = false;
            publisher.send(&msg)?;
            msg.axle_position = 1;
            msg.state = true;
            publisher.send(&msg)?;

            while vl_rear.read_sample()?.distance < 1100 {}
            msg.state = false;
            publisher.send(&msg)?;
            msg.axle_position = 2;
            msg.state = true;
            publisher.send(&msg)?;

            std::thread::sleep(Duration::from_secs(1));
            msg.state = false;
            publisher.send(&msg)?;
        } else {
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}
