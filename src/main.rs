use std::cmp::Ordering;

use crate::rc_param::{PassiveComponent, Resistor};

mod rc_param;

fn prefixed_for_resistance(val: f64) -> (f64, String) {
    match val {
        x if (1.0..1000.0).contains(&x) => ((x * 10.0f64).round() / 10.0f64, "".to_string()),
        x if x * 10f64.powi(-6) >= 1.0 => (
            (x * 10f64.powi(-6) * 10.0f64).round() / 10.0f64,
            "M".to_string(),
        ),
        x if x * 10f64.powi(-3) >= 1.0 => (
            (x * 10f64.powi(-3) * 10.0f64).round() / 10.0f64,
            "k".to_string(),
        ),
        x if x * 10f64.powi(3) >= 1.0 => (
            (x * 10f64.powi(3) * 10.0f64).round() / 10.0f64,
            "m".to_string(),
        ),
        x => (x, "".to_string()),
    }
}

#[derive(Copy, Clone)]
struct Constraint {
    pub voltage: Voltage,
    pub max_current: f64,
}

trait RangedType {
    type Item;
    fn get_typical_value(&self) -> Self::Item;
    fn get_min(&self) -> Self::Item;
    fn get_max(&self) -> Self::Item;
}

#[derive(Debug, Copy, Clone)]
struct Voltage {
    pub value: f64,
    min: f64,
    max: f64,
}

impl RangedType for Voltage {
    type Item = f64;

    fn get_typical_value(&self) -> Self::Item {
        self.value
    }

    fn get_min(&self) -> Self::Item {
        self.min
    }

    fn get_max(&self) -> Self::Item {
        self.max
    }
}

impl Voltage {
    pub fn new_by_allowance(value: f64, allowance: f64) -> Self {
        Self {
            value,
            min: value * (1.0 - allowance),
            max: value * (1.0 + allowance),
        }
    }

    pub fn new_by_values(value: f64, min: f64, max: f64) -> Self {
        Self { value, min, max }
    }

    pub fn min(&self) -> f64 {
        self.min
    }
    pub fn max(&self) -> f64 {
        self.max
    }
}

#[derive(Copy, Clone, Debug)]
struct RangedValue<T>
where
    T: Copy,
{
    pub value: T,
    min: T,
    max: T,
}

impl<T> RangedType for RangedValue<T>
where
    T: Copy,
{
    type Item = T;

    fn get_typical_value(&self) -> Self::Item {
        self.value
    }

    fn get_min(&self) -> Self::Item {
        self.min
    }

    fn get_max(&self) -> Self::Item {
        self.max
    }
}

impl<T> RangedValue<T>
where
    T: Copy,
{
    pub fn new(
        value: <Self as RangedType>::Item,
        min: <Self as RangedType>::Item,
        max: <Self as RangedType>::Item,
    ) -> Self {
        Self { value, min, max }
    }
}

type Gain = RangedValue<f64>;

#[derive(Debug)]
struct CircuitParameters {
    pub r1: Resistor,
    pub r2: Resistor,
    pub vref: Voltage,
    pub vref_error: f64,
}

enum VrefSource {
    Vcc(Voltage),
    Regulator(Voltage),
}

fn find_combinations(
    constraint: Constraint,
    v_src: Voltage,
    resistors: Vec<Resistor>,
) -> Vec<CircuitParameters> {
    let r1_resistors = resistors.to_vec();
    let r2_resistors = resistors.to_vec();
    let t = std::time::Instant::now();
    let mut combinations = r1_resistors
        .iter()
        .flat_map(|r1| {
            let r1_v = r1.get_value();
            let r1_min = r1.min();
            let r1_max = r1.max();
            r2_resistors
                .iter()
                .filter(|&r2| {
                    let r2_v = r2.get_value();
                    let r2_min = r2.min();
                    let max_curr = v_src.value / (r1_min + r2_min);
                    let vref = r2_v / (r1_v + r2_v) * v_src.value;
                    max_curr <= constraint.max_current
                        && vref >= constraint.voltage.min()
                        && vref <= constraint.voltage.max()
                })
                .map(|r2| {
                    let r2_v = r2.get_value();
                    let r2_min = r2.min();
                    let r2_max = r2.max();
                    let r = r2_v / (r1_v + r2_v);
                    let vref = {
                        let v_max = (r2_max / (r1_min + r2_max)) * v_src.max();
                        let v_min = (r2_min / (r1_max + r2_min)) * v_src.min();
                        Voltage::new_by_values(r * v_src.value, v_min, v_max)
                    };
                    let err = vref.value - constraint.voltage.value;
                    CircuitParameters {
                        r1: *r1,
                        r2: *r2,
                        vref,
                        vref_error: err,
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    dbg!(t.elapsed());
    combinations.sort_unstable_by(|a, b| {
        let x = a.vref_error.powi(2);
        let y = b.vref_error.powi(2);
        if x > y {
            Ordering::Greater
        } else if x < y {
            Ordering::Less
        } else if a.r1.get_value() + a.r2.get_value() > b.r1.get_value() + b.r2.get_value() {
            Ordering::Greater
        } else if a.r1.get_value() + a.r2.get_value() < b.r1.get_value() + b.r2.get_value() {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    });
    combinations
}

fn main() {
    let constraint = Constraint {
        voltage: Voltage::new_by_values(2.0, 0.5, 4.0),
        max_current: 5e-4,
    };
    let vcc: Voltage = Voltage::new_by_allowance(5.0f64, 5.0f64 * 0.050f64);
    let i_out = 1.9f64;

    let resistor_tolerance = 0.01;
    let resistors = rc_param::get_resistor_list(resistor_tolerance);

    let mut combinations = find_combinations(constraint, vcc, resistors);

    let r_rs = Resistor::new(0.47, resistor_tolerance);
    let gain = Gain::new(1.0 / 5.0, 1.0 / 5.2, 1.0 / 4.8);
    let k = RangedValue::new(
        gain.get_typical_value() / r_rs.get_value(),
        gain.get_min() / r_rs.max(),
        gain.get_max() / r_rs.min(),
    );

    combinations.sort_unstable_by(|a, b| {
        let i_a = k.get_typical_value() * a.vref.value;
        let i_b = k.get_typical_value() * b.vref.value;
        i_a.partial_cmp(&i_b).unwrap()
    });
    combinations.reverse();

    for x in combinations
        .iter()
        .filter(|&params| params.vref.max() <= constraint.voltage.max())
        .filter(|&params| k.get_max() * params.vref.max() <= i_out)
        .filter(|&params| params.r1.min() + params.r2.min() >= 10e3)
        .filter(|&params| params.r1.max() + params.r2.max() <= 120e3)
        .take(10)
        .collect::<Vec<_>>()
    {
        let (r1, r1_prefix) = prefixed_for_resistance(x.r1.get_value());
        println!("R1: {r1} {r1_prefix}Ω");
        let (r2, r2_prefix) = prefixed_for_resistance(x.r2.get_value());
        println!("R2: {r2} {r2_prefix}Ω");
        println!("Vref: {}", x.vref.value);
        println!("Vref Range: {}, {}", x.vref.min(), x.vref.max());
        println!("Iout: {}", k.get_typical_value() * x.vref.value);
        println!(
            "Iout Range: {}, {}",
            k.get_min() * x.vref.min(),
            k.get_max() * x.vref.max()
        );
        println!(
            "Iout Range (typ. Vref): {}, {}",
            k.get_min() * x.vref.get_typical_value(),
            k.get_max() * x.vref.get_typical_value()
        );
        println!("----------");
    }
}
