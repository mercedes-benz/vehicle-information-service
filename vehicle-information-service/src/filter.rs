// SPDX-License-Identifier: MIT

use crate::api_type::Filters;
use serde_json::{Number, Value};
use std::cmp::{Ord, Ordering};
use std::ops::Sub;
use std::time::{Duration, SystemTime};

#[cfg(test)]
mod tests {
    use crate::api_type::{FilterRange, Filters};
    use crate::filter;
    use crate::filter::{
        interval, is_in_filter_range, is_min_change, value_as_number, SerdeNumber,
    };
    use serde_json::{json, Value};
    use std::time::{Duration, SystemTime};

    #[test]
    fn value_as_number_ok_when_int() {
        assert!(value_as_number(&Value::Number(100.into())).is_ok());
        assert!(value_as_number(&Value::Number(From::from(-100))).is_ok());
    }

    #[test]
    fn value_as_number_err_when_not_int() {
        assert!(value_as_number(&Value::Bool(true)).is_err());
        assert!(value_as_number(&Value::Null).is_err());
    }

    #[test]
    fn is_in_filter_range_below_true_when_below_below() {
        let fr = FilterRange {
            below: Some(110.into()),
            above: None,
        };
        assert_eq!(
            Ok(true),
            is_in_filter_range(
                &json!(100),
                &Filters {
                    interval: None,
                    range: Some(fr),
                    min_change: None,
                },
            )
        );
    }

    #[test]
    fn is_in_filter_range_below_false_when_above_below() {
        let fr = FilterRange {
            below: Some(10.into()),
            above: None,
        };
        assert_eq!(
            Ok(false),
            is_in_filter_range(
                &json!(100),
                &Filters {
                    interval: None,
                    range: Some(fr),
                    min_change: None,
                },
            )
        );
    }

    #[test]
    fn is_in_filter_range_above_true_when_above_above() {
        let fr = FilterRange {
            below: None,
            above: Some(100.into()),
        };
        assert_eq!(
            Ok(true),
            is_in_filter_range(
                &json!(110),
                &Filters {
                    interval: None,
                    range: Some(fr),
                    min_change: None,
                },
            )
        );
    }

    #[test]
    fn is_in_filter_range_above_false_when_below_above() {
        let fr = FilterRange {
            below: None,
            above: Some(110.into()),
        };
        assert_eq!(
            Ok(false),
            is_in_filter_range(
                &json!(100),
                &Filters {
                    interval: None,
                    range: Some(fr),
                    min_change: None,
                },
            )
        );
    }

    #[test]
    fn is_in_filter_range_true_when_no_filter_set() {
        let fr = FilterRange {
            below: None,
            above: None,
        };
        assert_eq!(
            Ok(true),
            is_in_filter_range(
                &json!(100),
                &Filters {
                    interval: None,
                    range: Some(fr),
                    min_change: None,
                },
            )
        );
    }

    #[test]
    fn is_in_filter_range_true_when_above_and_below_set() {
        let fr = FilterRange {
            below: Some(120.into()),
            above: Some(100.into()),
        };
        assert_eq!(
            Ok(true),
            is_in_filter_range(
                &json!(110),
                &Filters {
                    interval: None,
                    range: Some(fr),
                    min_change: None,
                },
            )
        );
    }

    #[test]
    fn is_in_filter_range_true_when_above_and_below_set_float_signal() {
        let fr = FilterRange {
            below: Some(120.into()),
            above: Some(100.into()),
        };
        assert_eq!(
            Ok(true),
            is_in_filter_range(
                &json!(110.0),
                &Filters {
                    interval: None,
                    range: Some(fr),
                    min_change: None,
                },
            )
        );
    }

    #[test]
    fn is_in_filter_range_false_when_above_and_below_set_float_signal() {
        let fr = FilterRange {
            below: Some(120.into()),
            above: Some(115.into()),
        };
        assert_eq!(
            Ok(false),
            is_in_filter_range(
                &json!(110.0),
                &Filters {
                    interval: None,
                    range: Some(fr),
                    min_change: None,
                },
            )
        );
    }

    #[test]
    fn is_min_change_true_when_greater_or_equal() {
        let f = Filters {
            interval: None,
            range: None,
            min_change: Some(5.into()),
        };
        assert_eq!(
            Ok(true),
            is_min_change(
                &json!(1),
                &Some((SystemTime::now(), Value::Number(100.into()))),
                &f,
            )
        );
        assert_eq!(
            Ok(true),
            is_min_change(
                &json!(105),
                &Some((SystemTime::now(), Value::Number(100.into()))),
                &f,
            )
        );
    }

    #[test]
    fn is_min_change_false_when_smaller() {
        let f = Filters {
            interval: None,
            range: None,
            min_change: Some(5.into()),
        };
        assert_eq!(
            Ok(false),
            is_min_change(
                &json!(100),
                &Some((SystemTime::now(), Value::Number(101.into()))),
                &f,
            )
        );
    }

    #[test]
    fn matches_filter_true_when_changed() {
        let f = Some(Filters {
            interval: None,
            range: None,
            min_change: None,
        });
        assert_eq!(
            Ok(true),
            filter::matches(
                &Value::Number(00.into()),
                &Some((SystemTime::now(), Value::Number(101.into()))),
                &f,
            )
        );
    }

    #[test]
    fn matches_filter_false_when_not_changed() {
        let f = Some(Filters {
            interval: None,
            range: None,
            min_change: None,
        });
        assert_eq!(
            Ok(false),
            filter::matches(
                &Value::Number(101.into()),
                &Some((SystemTime::now(), Value::Number(101.into()))),
                &f,
            )
        );
    }

    #[test]
    fn matches_filter_false_when_not_min_change() {
        let f = Some(Filters {
            interval: None,
            range: None,
            min_change: Some(5.into()),
        });
        assert_eq!(
            Ok(false),
            filter::matches(
                &Value::Number(101.into()),
                &Some((SystemTime::now(), Value::Number(104.into()))),
                &f,
            )
        );
    }

    #[test]
    fn matches_filter_true_when_min_change() {
        let f = Some(Filters {
            interval: None,
            range: None,
            min_change: Some(5.into()),
        });
        assert_eq!(
            Ok(true),
            filter::matches(
                &Value::Number(100.into()),
                &Some((SystemTime::now(), Value::Number(106.into()))),
                &f,
            )
        );
    }

    #[test]
    fn matches_filter_true_when_float_changed() {
        let f = Some(Filters {
            interval: None,
            range: None,
            min_change: Some(5.into()),
        });
        assert_eq!(
            Ok(true),
            filter::matches(&json!(100.1), &Some((SystemTime::now(), json!(106.7))), &f,)
        );
    }

    #[test]
    fn matches_filter_false_when_float_unchanged() {
        let f = Some(Filters {
            interval: None,
            range: None,
            min_change: Some(5.into()),
        });
        assert_eq!(
            Ok(false),
            filter::matches(&json!(100.1), &Some((SystemTime::now(), json!(100.5))), &f,)
        );
    }

    #[test]
    fn matches_filter_true_when_string_changed() {
        let f = Some(Filters {
            interval: None,
            range: None,
            min_change: None,
        });
        assert_eq!(
            Ok(true),
            filter::matches(
                &Value::String("a".to_owned()),
                &Some((SystemTime::now(), Value::String("b".to_owned()))),
                &f,
            )
        );
    }

    #[test]
    fn matches_filter_false_when_string_unchanged() {
        let f = Some(Filters {
            interval: None,
            range: None,
            min_change: None,
        });
        assert_eq!(
            Ok(false),
            filter::matches(
                &Value::String("a".to_owned()),
                &Some((SystemTime::now(), Value::String("a".to_owned()))),
                &f,
            )
        );
    }

    #[test]
    fn matches_filter_false_when_no_filter_set_and_value_not_changed() {
        let last_val = &Some((SystemTime::now(), Value::Number(100.into())));
        let filter = None;
        assert_eq!(
            Ok(false),
            filter::matches(&Value::Number(100.into()), last_val, &filter,)
        );
    }

    #[test]
    fn matches_filter_true_when_no_filter_set_and_value_changed() {
        let last_val = &Some((SystemTime::now(), Value::Number(100.into())));
        let filter = None;
        assert_eq!(
            Ok(true),
            filter::matches(&Value::Number(101.into()), last_val, &filter,)
        );
    }

    #[test]
    fn interval_true_when_duration_to_last_val_greater_than_interval() {
        let f = Filters {
            interval: Some(100),
            range: None,
            min_change: None,
        };
        let now = SystemTime::now();
        let later = now.clone() + Duration::from_secs(10);
        assert!(interval(
            later,
            &Some((now, Value::String("a".to_owned()))),
            &f,
        ));
    }

    #[test]
    fn interval_false_when_duration_to_last_val_smaller_than_interval() {
        let f = Filters {
            interval: Some(1000000),
            range: None,
            min_change: None,
        };
        let now = SystemTime::now();
        let later = now.clone() + Duration::from_millis(10);
        assert!(!interval(later, &Some((now, Value::Null)), &f));
    }

    #[test]
    fn serde_number_eq() {
        assert_eq!(SerdeNumber(1.into()), SerdeNumber(1.into()));
        assert!(SerdeNumber(1.into()) != SerdeNumber(100.into()));
        let u: i64 = -100;
        assert!(SerdeNumber(u.into()).abs() == SerdeNumber(100.into()));
        assert!((SerdeNumber(u.into()) - SerdeNumber(u.into())).abs() == SerdeNumber(0.into()));
        assert!(SerdeNumber(u.into()).abs() >= SerdeNumber(100.into()));
        assert!(SerdeNumber(u.into()).abs() > SerdeNumber(50.into()));
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum Error {
    ValueIsNotANumber,
}

///
/// Does the val match the filter criteria
/// Returns:
/// Ok(true) : E.g. value changed sufficiently or there was no filter set
/// Ok(false) : Did not reach change threshold
/// Err(...): Occurs when the value is not an integer, filters only work for ints
///
pub fn matches(
    val: &Value,
    last_value: &Option<(SystemTime, Value)>,
    filters_opt: &Option<Filters>,
) -> Result<bool, Error> {
    debug!(
        "Matches filter val {:?}, last value {:?}, filters {:?}",
        val, last_value, filters_opt
    );

    let changed_exp = last_value.as_ref().map_or(true, |v| val != &v.1);

    let filters_exp = if let Some(filters) = filters_opt {
        let interval_exp = interval(SystemTime::now(), last_value, filters);

        let range_exp = is_in_filter_range(&val, filters)?;
        let min_change_exp = is_min_change(&val, last_value, filters)?;
        debug!(
            "Matches filter val {:?}, last value {:?}, filters {:?}, changed_exp? {}, range_exp? {}, min_change_exp? {}",
            val, last_value, filters, changed_exp, range_exp, min_change_exp,
        );

        let range_min_exp = range_exp && min_change_exp;
        interval_exp && range_min_exp
    } else {
        // filter exp is None
        true
    };

    Ok(changed_exp && filters_exp)
}

///
/// Extract a Number from a JSON Value or return Error if not possible.
///
fn value_as_number(val: &Value) -> Result<SerdeNumber, Error> {
    if let Value::Number(ref num) = *val {
        Ok(SerdeNumber(num.clone()))
    } else {
        Err(Error::ValueIsNotANumber)
    }
}

fn interval(now: SystemTime, last_value: &Option<(SystemTime, Value)>, filters: &Filters) -> bool {
    last_value.as_ref().map_or(true, |v| {
        now.duration_since(v.0)
            .ok()
            .as_ref()
            .and_then(|d| filters.interval.map(|i| Duration::from_millis(i) <= *d))
            .unwrap_or(true)
    })
}

///
/// Below or above filter
///
fn is_in_filter_range(val: &Value, filters: &Filters) -> Result<bool, Error> {
    if let Some(ref range) = filters.range {
        let num = value_as_number(val)?;
        let below = range.clone().below.map_or(true, |b| num <= SerdeNumber(b));
        let above = range.clone().above.map_or(true, |a| num >= SerdeNumber(a));
        Ok(below && above)
    } else {
        // No range filter
        Ok(true)
    }
}

///
/// Changed by at least x compared to last value.
/// Returns None if there is no last value.
///
fn is_min_change(
    val: &Value,
    last_value: &Option<(SystemTime, Value)>,
    filters: &Filters,
) -> Result<bool, Error> {
    debug!("Last value {:?}, new value {:?}", last_value, val);

    if let Some(ref filter_min_change) = filters.min_change {
        if let Some((_time, value)) = last_value {
            let num = value_as_number(val)?;
            let as_number = value_as_number(value)?;
            return Ok((as_number - num).abs() >= SerdeNumber(filter_min_change.clone()));
        }
    }

    // no filter min change in subscription or no last value yet
    Ok(true)
}

#[derive(Clone, Debug)]
struct SerdeNumber(Number);

impl SerdeNumber {
    fn abs(self) -> Self {
        if self.0.is_u64() {
            self
        } else if self.0.is_i64() {
            Self(self.0.as_i64().unwrap_or(9999).abs().into())
        } else {
            Self(
                Number::from_f64(self.0.as_f64().unwrap_or_default().abs()).unwrap_or_else(|| {
                    Number::from_f64(0.0).expect("Unexpected number conversion error")
                }),
            )
        }
    }
}

impl PartialEq for SerdeNumber {
    fn eq(&self, other: &Self) -> bool {
        if self.0.is_u64() && other.0.is_u64() {
            self.0.as_u64() == other.0.as_u64()
        } else if self.0.is_i64() && other.0.is_i64() {
            self.0.as_i64() == other.0.as_i64()
        } else {
            self.0.as_f64() == other.0.as_f64()
        }
    }
}

impl Eq for SerdeNumber {}

impl PartialOrd for SerdeNumber {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.0.is_u64() && other.0.is_u64() {
            self.0
                .as_u64()
                .unwrap_or_default()
                .partial_cmp(&other.0.as_u64().unwrap_or_default())
        } else if self.0.is_i64() && other.0.is_i64() {
            self.0
                .as_i64()
                .unwrap_or_default()
                .partial_cmp(&other.0.as_i64().unwrap_or_default())
        } else {
            self.0
                .as_f64()
                .as_ref()
                .and_then(|x| other.0.as_f64().as_ref().and_then(|y| x.partial_cmp(y)))
        }
    }
}

impl Ord for SerdeNumber {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

impl Sub for SerdeNumber {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn sub(self, other: Self) -> Self {
        if self.0.is_u64() && other.0.is_u64() {
            Self(
                (self
                    .0
                    .as_u64()
                    .unwrap_or_default()
                    .wrapping_sub(other.0.as_u64().unwrap_or_default()))
                .into(),
            )
        } else if self.0.is_i64() && other.0.is_i64() {
            Self(
                (self
                    .0
                    .as_i64()
                    .unwrap_or_default()
                    .wrapping_sub(other.0.as_i64().unwrap_or_default()))
                .into(),
            )
        } else {
            Self(
                Number::from_f64(
                    self.0
                        .as_f64()
                        .unwrap_or_default()
                        .abs()
                        .sub(other.0.as_f64().unwrap_or_default().abs()),
                )
                .unwrap_or_else(|| 0.into()),
            )
        }
    }
}
