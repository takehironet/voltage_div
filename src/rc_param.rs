use std::ops::Range;

const E24: [f64; 24] = [
    1.0, 1.1, 1.2, 1.3, 1.5, 1.6, 1.8, 2.0, 2.2, 2.4, 2.7, 3.0, 3.3, 3.6, 3.9, 4.3, 4.7, 5.1, 5.6,
    6.2, 6.8, 7.5, 8.2, 9.1,
];

#[allow(dead_code)]
#[repr(usize)]
enum Series {
    E24 = 1,
    E12 = 2,
    E6 = 4,
    E3 = 8,
}

fn get_e_series_values(series: Series) -> Vec<f64> {
    E24.iter()
        .step_by(series as usize)
        .copied()
        .collect::<Vec<f64>>()
}

pub trait PassiveComponent {
    fn get_value(&self) -> f64;
    fn get_tolerance(&self) -> f64;
    fn max(&self) -> f64 {
        self.get_value() * (1.0 + self.get_tolerance())
    }
    fn min(&self) -> f64 {
        self.get_value() * (1.0 - self.get_tolerance())
    }
    fn new(value: f64, tolerance: f64) -> Self;
}

#[derive(Clone, Copy, Debug)]
pub struct Resistor {
    value: f64,
    tolerance: f64,
}

impl PassiveComponent for Resistor {
    fn get_value(&self) -> f64 {
        self.value
    }

    fn get_tolerance(&self) -> f64 {
        self.tolerance
    }

    fn new(value: f64, tolerance: f64) -> Self {
        Resistor { value, tolerance }
    }
}

impl Resistor {
    #[allow(dead_code)]
    fn new_from_value(value: f64) -> Self {
        Self::new(value, 0.05)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Capacitor {
    value: f64,
    tolerance: f64,
}

impl PassiveComponent for Capacitor {
    fn get_value(&self) -> f64 {
        self.value
    }
    fn get_tolerance(&self) -> f64 {
        self.tolerance
    }
    fn new(value: f64, tolerance: f64) -> Self {
        Capacitor { value, tolerance }
    }
}

impl Capacitor {
    #[allow(dead_code)]
    fn new_from_value(value: f64) -> Self {
        Self::new(value, 0.20)
    }
}

#[allow(dead_code)]
pub fn get_capacitor_list(tolerance: f64) -> Vec<Capacitor> {
    let expressions_for_e6 = 1u8..6u8;
    let expressions_for_e3 = 6u8..9u8;
    let e6 = get_e_series_values(Series::E6);
    let e3 = get_e_series_values(Series::E3);

    let value_to_pico = |v: f64, exp: i32| -> f64 { v * 10f64.powi(exp - 12) };

    let generate_values_list =
        |expressions: Range<u8>, series_values: Vec<f64>| -> Vec<Capacitor> {
            expressions
                .flat_map(|exp| {
                    series_values
                        .iter()
                        .map(|v| Capacitor::new(value_to_pico(*v, exp as i32), tolerance))
                        .collect::<Vec<Capacitor>>()
                })
                .collect()
        };

    let values_e6: Vec<Capacitor> = generate_values_list(expressions_for_e6, e6);
    let values_e3: Vec<Capacitor> = generate_values_list(expressions_for_e3, e3);
    vec![values_e6, values_e3]
        .iter()
        .flatten()
        .cloned()
        .collect::<Vec<Capacitor>>()
}

#[allow(dead_code)]
pub fn get_resistor_list(tolerance: f64) -> Vec<Resistor> {
    let expressions = 0u8..6u8;
    let e24 = get_e_series_values(Series::E24);

    expressions
        .flat_map(|exp| {
            e24.iter()
                .map(|v| Resistor::new(*v * 10.0f64.powi(exp as i32), tolerance))
                .collect::<Vec<Resistor>>()
        })
        .collect()
}
