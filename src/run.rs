use crate::direction::Direction;
use crate::robot::Movable;
use crate::robot::Robot;
use crate::world::wall::Wall;

pub fn run() {
    let mut robot = Robot::new(1);
    println!("{robot}");

    let _ = robot
        .move_robot(Direction::Up(5))
        .inspect_err(|err| println!("Could not move robot: {err:?}"));
    let _ = robot.move_robot(Direction::Right);
    if robot.move_robot(Direction::Up(2)).is_ok() {
        println!("Movement was possible");
    }
    robot.move_robot(Direction::Right).unwrap();
    println!("{robot}");
    robot.charge(112);
    println!("{robot}");
    robot.move_robot(Direction::Down).unwrap();
    robot.move_robot(Direction::Left).unwrap();
    println!("{robot}");

    let mut robot2 = Robot::new(2);
    robot2.move_robot(Direction::Up(2)).unwrap();
    robot2.move_robot(Direction::Right).unwrap();
    robot2.move_robot(Direction::Right).unwrap();
    let wc = robot2.would_collide(Direction::Left, &robot);
    if !wc {
        let _ = robot2.move_robot(Direction::Left);
    }
    println!("{robot}");
}
