use crate::{Counter, Report};

pub struct Pointers {
    counter: Counter,
    positions: [(u32, u32); 10],
    positions_length: usize,
    mappings: [usize; 10],
    events: [Report; 10],
}

const DETACH_THRESHOLD: u32 = 10000;

impl Pointers {
    pub fn new() -> Pointers {
        Pointers {
            counter: Counter::default(),
            positions: [(0, 0); 10],
            positions_length: 0,
            mappings: [0; 10],
            events: [Report::None; 10],
        }
    }

    pub fn update(&mut self, mut new: [(u32, u32); 10], new_length: usize) {
        let old = self.positions;
        let old_length = self.positions_length;
        assert!(new_length <= 10);
        assert!(old_length <= 10);

        self.events.fill(Report::None);

        let distances = {
            let mut distances = [0u32; 100];
            let mut distance_i = 99;
            let mut new_i = 9;
            while {
                let new = new[new_i];
                let mut old_i = 9;
                while {
                    let dist = point_dist(old[old_i], new);
                    distances[distance_i] = dist;
                    old_i != 0
                } { old_i -= 1; distance_i -= 1; }
                new_i != 0
            } { new_i -= 1; distance_i -= 1; }
            distances
        };

        let mut mappings = [0usize; 10];
        let mut new_pending = Set10Iter::new(new_length);
        let mut old_pending = Set10Iter::new(old_length);
        'stop: loop {
            let mut min_dist = u32::MAX;
            let mut min_new = 0;
            let mut min_old = 0;

            let mut new_p = new_pending.next(0);
            if new_p == 0 { break 'stop; }
            while {
                let new_offset = (new_p - 1) * 10;

                let mut old_p = old_pending.next(0);
                if old_p == 0 { break 'stop; }
                while {
                    let dist = distances[new_offset + old_p - 1];
                    if dist < min_dist {
                        min_dist = dist;
                        min_new = new_p;
                        min_old = old_p;
                    }

                    old_p = old_pending.next(old_p);
                    old_p != 0
                } {}

                new_p = new_pending.next(new_p);
                new_p != 0
            } {}

            debug_assert!(min_dist != u32::MAX);
            if min_dist > DETACH_THRESHOLD { break; }

            new_pending.remove(min_new);
            old_pending.remove(min_old);

            let old_i = min_old - 1;
            let mapped = self.mappings[old_i];
            let new_i = min_new - 1;
            mappings[new_i] = mapped;
            let mut pos = new[new_i];
            let old_pos = old[old_i];
            // Ignore anything less than 16 units away
            if min_dist > 256 {
                fn isqrt(num: i32) -> i32 {
                    let mut result = 3;
                    let mut square = 9;
                    while square < num {
                        result += 1;
                        square += result - 1;
                    }
                    result
                }

                let dx = pos.0 as i32 - old_pos.0 as i32;
                let dy = pos.1 as i32 - old_pos.1 as i32;
                let norm = isqrt((dx * dx) + (dy * dy));
                let dx_clip = (dx * 16) / norm;
                let dy_clip = (dy * 16) / norm;
                pos.0 = (pos.0 as i32 - dx_clip) as u32;
                pos.1 = (pos.1 as i32 - dy_clip) as u32;
            } else {
                pos = old_pos;
            }
            new[new_i] = pos;
            // TODO Further smooth out the jitter
            self.events[mapped] = Report::Move(pos);
        } {}

        {
            let mut old_p = old_pending.next(0);
            while old_p != 0 {
                self.events[self.mappings[old_p - 1]] = Report::Up;
                old_p = old_pending.next(old_p);
            }

            let mut new_p = new_pending.next(0);
            let mut events_insert_iter = EventsInsertIter::new(&mut self.events);
            while new_p != 0 {
                let new_i = new_p - 1;
                let pos = new[new_i];
                mappings[new_i] = events_insert_iter.insert(pos);
                new_p = new_pending.next(new_p);
            }
        }

        self.positions = new;
        self.positions_length = new_length;
        self.mappings = mappings;
    }

    pub fn events(&self) -> &[Report; 10] {
        &self.events
    }

    pub fn events_and_counter(&mut self) -> (&[Report; 10], &mut Counter) {
        (&self.events, &mut self.counter)
    }
}

#[derive(Eq, PartialEq)]
enum EventsInsertIterStage {
    None,
    Up,
}

impl EventsInsertIterStage {
    fn advance(&mut self) {
        *self = match *self {
            EventsInsertIterStage::None => EventsInsertIterStage::Up,
            _ => panic!(),
        }
    }
}

struct EventsInsertIter<'a> {
    events: &'a mut [Report; 10],
    head: usize,
    stage: EventsInsertIterStage,
}

impl <'a> EventsInsertIter<'a> {
    #[inline]
    fn new(events: &'a mut [Report; 10]) -> EventsInsertIter<'a> {
        EventsInsertIter {
            events,
            head: 0,
            stage: EventsInsertIterStage::None,
        }
    }

    fn advance_head(&mut self) {
        self.head += 1;
        if self.head == 10 {
            self.head = 0;
            self.stage.advance();
        }
    }

    #[inline]
    fn insert(&mut self, pos: (u32, u32)) -> usize {
        while match self.events[self.head] {
            Report::None => self.stage != EventsInsertIterStage::None,
            Report::Up => self.stage != EventsInsertIterStage::Up,
            _ => true,
        } {
            self.advance_head();
        }
        let result = self.head;
        self.events[result] = match self.stage {
            EventsInsertIterStage::None => Report::Down(pos),
            EventsInsertIterStage::Up => Report::UpDown(pos),
        };
        self.advance_head();
        result
    }
}

struct Set10Iter {
    pending: [usize; 11],
}

impl Set10Iter {
    #[inline]
    fn new(length: usize) -> Set10Iter {
        let mut pending =  [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 0];
        pending[length] = 0;
        Set10Iter {
            pending,
        }
    }

    #[inline]
    fn next(&self, pos: usize) -> usize {
        self.pending[pos]
    }

    #[inline]
    fn remove(&mut self, mut pos: usize) {
        let next = self.pending[pos];
        let cur = pos;
        self.pending[cur] = 0;
        while {
            pos -= 1;
            self.pending[pos] == 0
        } {}
        debug_assert_eq!(self.pending[pos], cur);
        self.pending[pos] = next;
    }
}

#[inline]
fn sq_diff(a: u32, b: u32) -> u32 {
    let diff = if a < b {
        b - a
    } else {
        a - b
    };
    diff * diff
}

#[inline]
fn point_dist(a: (u32, u32), b: (u32, u32)) -> u32 {
    sq_diff(a.0, b.0) + sq_diff(a.1, b.1)
}
