#[derive(Debug)]
pub(crate) enum Constraint {
    Min(i64),
    Max(i64),
    NotEmpty,
    Optional,
}

pub(crate) fn validate_bool_constraints(key: &str, constraints: &[Constraint]) {
    for con in constraints {
        match con {
            Constraint::Optional => continue,
            _ => {
                panic!("Invalid constraint for {key}. Only Optional is acceptable for boolean type")
            }
        }
    }
}

pub(crate) fn validate_num_constraints(key: &str, constraints: &[Constraint]) {
    for con in constraints {
        match con {
            Constraint::NotEmpty => {
                panic!("Invalid constraint NotEmpty for {key}")
            }
            _ => continue,
        }
    }
}
