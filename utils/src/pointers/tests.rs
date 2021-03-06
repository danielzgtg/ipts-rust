use crate::Pointers;
use crate::Report;
use std::time::Instant;

#[test]
fn it_works() {
    let start = Instant::now();
    let mut pointers = Pointers::new();
    let mut expected = [Report::None; 10];
    assert_eq!(pointers.events(), &expected);
    read_reports(&pointers);

    let mut positions: [(u32, u32); 10] = [(0, 0); 10];
    pointers.update(positions, 0);
    assert_eq!(pointers.events(), &expected);
    read_reports(&pointers);

    positions[0] = (1, 1);
    pointers.update(positions, 0);
    assert_eq!(pointers.events(), &expected);
    read_reports(&pointers);

    pointers.update(positions, 1);
    expected[0] = Report::Down((1, 1));
    assert_eq!(pointers.events(), &expected);
    read_reports(&pointers);

    pointers.update(positions, 1);
    expected[0] = Report::Move((1, 1));
    assert_eq!(pointers.events(), &expected);
    read_reports(&pointers);

    positions[0] = (1, 2);
    pointers.update(positions, 1);
    expected[0] = Report::Move((1, 1));
    assert_eq!(pointers.events(), &expected);
    read_reports(&pointers);

    positions[0] = (1, 20);
    pointers.update(positions, 1);
    expected[0] = Report::Move((1, 20));
    assert_eq!(pointers.events(), &expected);
    read_reports(&pointers);

    positions[0] = (1, 50);
    pointers.update(positions, 1);
    expected[0] = Report::Move((1, 50));
    assert_eq!(pointers.events(), &expected);
    read_reports(&pointers);

    positions[0] = (1, 5);
    positions[1] = (100, 500);
    pointers.update(positions, 2);
    expected[0] = Report::Move((1, 5));
    expected[1] = Report::Down((100, 500));
    assert_eq!(pointers.events(), &expected);
    read_reports(&pointers);

    pointers.update(positions, 2);
    expected[0] = Report::Move((1, 5));
    expected[1] = Report::Move((100, 500));
    assert_eq!(pointers.events(), &expected);
    read_reports(&pointers);

    positions.swap(0, 1);
    pointers.update(positions, 2);
    assert_eq!(pointers.events(), &expected);
    read_reports(&pointers);

    pointers.update(positions, 2);
    assert_eq!(pointers.events(), &expected);
    read_reports(&pointers);

    positions.swap(0, 1);
    pointers.update(positions, 2);
    assert_eq!(pointers.events(), &expected);
    read_reports(&pointers);

    positions[2] = (50, 40);
    pointers.update(positions, 2);
    assert_eq!(pointers.events(), &expected);
    read_reports(&pointers);

    pointers.update(positions, 3);
    expected[2] = Report::Down((50, 40));
    assert_eq!(pointers.events(), &expected);
    read_reports(&pointers);

    positions[1] = positions[2];
    pointers.update(positions, 2);
    expected[1] = Report::Up;
    expected[2] = Report::Move((50, 40));
    assert_eq!(pointers.events(), &expected);
    read_reports(&pointers);

    pointers.update(positions, 2);
    expected[1] = Report::None;
    assert_eq!(pointers.events(), &expected);
    read_reports(&pointers);

    positions[2] = (800, 900);
    pointers.update(positions, 3);
    expected[1] = Report::Down((800, 900));
    assert_eq!(pointers.events(), &expected);
    read_reports(&pointers);

    positions[0] = positions[1];
    pointers.update(positions, 1);
    expected[0] = Report::Up;
    expected[1] = Report::Up;
    assert_eq!(pointers.events(), &expected);
    read_reports(&pointers);

    positions[0] = (71, 61);
    pointers.update(positions, 1);
    expected[0] = Report::None;
    expected[1] = Report::None;
    expected[2] = Report::Move((71, 61));
    assert_eq!(pointers.events(), &expected);
    read_reports(&pointers);

    positions[0] = (52, 42);
    positions[1] = (11, 22);
    pointers.update(positions, 2);
    expected[0] = Report::Down((11, 22));
    expected[2] = Report::Move((52, 42));
    assert_eq!(pointers.events(), &expected);
    read_reports(&pointers);
    println!("Elapsed {:?}", Instant::now() - start);
}

fn read_reports(_pointers: &Pointers) {
    // for event in pointers.events() {
    //     print!("{:?} ", event);
    // }
    // println!();
}
