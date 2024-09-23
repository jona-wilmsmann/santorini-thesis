pub fn ns_formatter(ns: &usize) -> String {
    return if *ns < 1000 {
        format!("{} ns", ns)
    } else if *ns < 1_000_000 {
        number_to_3_relevant_digits(*ns as f64 / 1000.0, " μs")
    } else if *ns < 1_000_000_000 {
        number_to_3_relevant_digits(*ns as f64 / 1_000_000.0, " ms")
    } else {
        number_to_3_relevant_digits(*ns as f64 / 1_000_000_000.0, " s")
    };
}

pub fn value_formatter(value: &usize) -> String {
    if *value < 1000 {
        format!("{}", value)
    } else if *value < 1_000_000 {
        let formatted = *value as f64 / 1000.0;
        number_to_3_relevant_digits(formatted, "k")
    } else if *value < 1_000_000_000 {
        let formatted = *value as f64 / 1_000_000.0;
        number_to_3_relevant_digits(formatted, "M")
    } else if *value < 1_000_000_000_000 {
        let formatted = *value as f64 / 1_000_000_000.0;
        number_to_3_relevant_digits(formatted, "B")
    } else {
        let formatted = *value as f64 / 1_000_000_000_000.0;
        number_to_3_relevant_digits(formatted, "T")
    }
}

// Expects a number <1000
fn number_to_3_relevant_digits(number: f64, suffix: &str) -> String {
    let mut number_string;
    if number > 100.0 {
        number_string = format!("{:.0}", number);
    } else if number > 10.0 {
        number_string = format!("{:.1}", number);
    } else {
        number_string = format!("{:.2}", number);
    }
    while number_string.ends_with("0") {
        number_string.pop();
    }
    if number_string.ends_with(".") {
        number_string.pop();
    }
    return format!("{}{}", number_string, suffix);
}