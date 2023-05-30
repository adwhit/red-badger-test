use std::{collections::BTreeSet, fmt::Display};

fn main() -> Result<()> {
    let mut args = std::env::args();
    let _ = args.next();
    let input = args.next().ok_or_else(|| "no input file specified")?;
    let input = std::fs::read_to_string(input)?;
    run_input(&input)?;
    Ok(())
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn parse_input(s: &str) -> Result<Input> {
    let mut lines = s.lines();
    let Some(world) = lines.next() else {
        return Err("no grid found")?;
    };
    let world_size = world
        .split_whitespace()
        .filter_map(|val| val.parse::<i8>().ok())
        .collect::<Vec<_>>();
    if world_size.len() != 2 {
        return Err(format!("could not parse world size: {world}"))?;
    }
    let mut robot_inputs = Vec::new();
    loop {
        let Some(ri) = parse_robot_input(&mut lines)? else { break };
        robot_inputs.push(ri);
    }
    Ok(Input {
        init_world: (world_size[0], world_size[1]),
        robot_inputs,
    })
}

fn parse_robot_input<'a>(mut s: impl Iterator<Item = &'a str>) -> Result<Option<RobotInput>> {
    let startpos = loop {
        match s.next() {
            None => return Ok(None),
            Some("") => continue,
            Some(s) => break s,
        }
    };
    let start: Vec<_> = startpos.split_whitespace().collect();
    if start.len() != 3 {
        return Err(format!("Could not parse robot start pos: {startpos}"))?;
    }
    let start_x = start[0].parse::<i8>()?;
    let start_y = start[1].parse::<i8>()?;
    let start_orientation = Orientation::parse(&start[2])?;
    let Some(cmdstr) = s.next() else {
        return Err("no commands found")?;
    };
    let commands = cmdstr
        .chars()
        .map(|val| Command::parse(val))
        .collect::<Result<Vec<_>>>()?;
    Ok(Some(RobotInput {
        start_x,
        start_y,
        start_orientation,
        commands,
    }))
}

struct RobotInput {
    start_x: i8,
    start_y: i8,
    start_orientation: Orientation,
    commands: Vec<Command>,
}

struct Input {
    init_world: (i8, i8),
    robot_inputs: Vec<RobotInput>,
}

impl Input {
    fn into_world(self) -> Result<(World, Vec<(Robot, Vec<Command>)>)> {
        let world = World::new(self.init_world.0, self.init_world.1)?;
        let robots = self
            .robot_inputs
            .into_iter()
            .map(|ri| {
                Robot::new(ri.start_x, ri.start_y, ri.start_orientation, &world)
                    .map(|r| (r, ri.commands))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok((world, robots))
    }
}

#[derive(Copy, Clone, Debug)]
enum Command {
    Forward,
    Left,
    Right,
}

impl Command {
    fn parse(i: char) -> Result<Self> {
        let cmd = match i {
            'F' => Command::Forward,
            'L' => Command::Left,
            'R' => Command::Right,
            _ => return Err(format!("not a valid command: '{i}'"))?,
        };
        Ok(cmd)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Orientation {
    North,
    East,
    South,
    West,
}

impl Display for Orientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Orientation::North => write!(f, "N"),
            Orientation::East => write!(f, "E"),
            Orientation::South => write!(f, "S"),
            Orientation::West => write!(f, "W"),
        }
    }
}

impl Orientation {
    fn delta(&self) -> (i8, i8) {
        match self {
            Orientation::North => (0, 1),
            Orientation::East => (1, 0),
            Orientation::South => (0, -1),
            Orientation::West => (-1, 0),
        }
    }

    fn right(&self) -> Self {
        use Orientation::*;
        match self {
            North => East,
            East => South,
            South => West,
            West => North,
        }
    }

    fn left(&self) -> Self {
        use Orientation::*;
        match self {
            North => West,
            West => South,
            South => East,
            East => North,
        }
    }
}

impl Orientation {
    fn parse(i: &str) -> Result<Self> {
        let cmd = match i {
            "N" => Orientation::North,
            "E" => Orientation::East,
            "S" => Orientation::South,
            "W" => Orientation::West,
            _ => return Err(format!("not a valid orientation: '{i}'"))?,
        };
        Ok(cmd)
    }
}

struct World {
    max_x: i8,
    max_y: i8,
    tombstones: BTreeSet<(i8, i8)>,
}

impl World {
    fn new(max_x: i8, max_y: i8) -> Result<Self> {
        if max_x > 50 || max_y > 50 {
            return Err(format!("Bad dimensions: {max_x} {max_y}"))?;
        }
        Ok(Self {
            max_x,
            max_y,
            tombstones: Default::default(),
        })
    }

    fn run(&mut self, mut robot: Robot, commands: &[Command]) -> Outcome {
        for c in commands {
            match c {
                Command::Forward => {
                    let (dx, dy) = robot.orientation.delta();
                    let new_x = robot.x + dx;
                    let new_y = robot.y + dy;
                    if self.tombstones.contains(&(new_x, new_y)) {
                        continue;
                    }
                    if new_x > self.max_x || new_y > self.max_y || new_x < 0 || new_y < 0 {
                        // illegal move
                        self.tombstones.insert((new_x, new_y));
                        return Outcome {
                            final_x: robot.x,
                            final_y: robot.y,
                            final_orientation: robot.orientation,
                            lost: true,
                        };
                    }
                    robot.x = new_x;
                    robot.y = new_y;
                }
                Command::Left => {
                    robot.orientation = robot.orientation.left();
                }
                Command::Right => {
                    robot.orientation = robot.orientation.right();
                }
            }
        }
        Outcome {
            final_x: robot.x,
            final_y: robot.y,
            final_orientation: robot.orientation,
            lost: false,
        }
    }
}

#[derive(PartialEq, Debug)]
struct Outcome {
    final_x: i8,
    final_y: i8,
    final_orientation: Orientation,
    lost: bool,
}

struct Robot {
    x: i8,
    y: i8,
    orientation: Orientation,
}

impl Robot {
    fn new(startx: i8, starty: i8, orientation: Orientation, world: &World) -> Result<Self> {
        if startx > world.max_x || starty > world.max_y {
            return Err(format!("Invalid start point for robot {startx} {starty}"))?;
        }
        Ok(Self {
            x: startx,
            y: starty,
            orientation,
        })
    }
}

fn run_input(input: &str) -> Result<()> {
    let input = parse_input(input)?;
    let (mut world, robots) = input.into_world()?;
    let outcomes: Vec<_> = robots.into_iter().map(|(r, c)| world.run(r, &c)).collect();
    for o in outcomes {
        print!("{} {} {}", o.final_x, o.final_y, o.final_orientation);
        if o.lost {
            print!(" LOST");
        }
        println!()
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_input() {
        let input = r#"5 3
1 1 E
RFRFRFRF

3 2 N
FRRFLLFFRRFLL

0 3 W
LLFFFLFLFL"#;
        let input = parse_input(input).unwrap();
        assert_eq!(input.robot_inputs.len(), 3);
        let (mut world, robots) = input.into_world().unwrap();
        let outcomes: Vec<_> = robots.into_iter().map(|(r, c)| world.run(r, &c)).collect();
        assert_eq!(
            outcomes,
            [
                Outcome {
                    final_x: 1,
                    final_y: 1,
                    final_orientation: Orientation::East,
                    lost: false
                },
                Outcome {
                    final_x: 3,
                    final_y: 3,
                    final_orientation: Orientation::North,
                    lost: true
                },
                Outcome {
                    final_x: 2,
                    final_y: 3,
                    final_orientation: Orientation::South,
                    lost: false
                },
            ]
        );
    }

    #[test]
    fn test_bad_input_errors() {
        assert!(parse_input("5 5\n1 2 3").is_err())
    }
}
