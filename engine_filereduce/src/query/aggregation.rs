use crate::query::ast::Aggregate;
use crate::row::Row;
use crate::row::Value;

#[derive(Debug, Clone)]
pub struct AggregateResult {
    pub count: Option<usize>,
    pub sum: Option<f64>,
    pub avg: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

pub fn execute_aggregates(rows: &[Row], aggregates: &[Aggregate]) -> AggregateResult {
    let mut result = AggregateResult {
        count: None,
        sum: None,
        avg: None,
        min: None,
        max: None,
    };

    for agg in aggregates {
        match agg {
            Aggregate::Count(field) => {
                let count = if field == "*" {
                    rows.len()
                } else {
                    rows.iter().filter(|row| row.get(field).is_some()).count()
                };
                result.count = Some(count);
            }
            Aggregate::Sum(field) => {
                let values: Vec<f64> = rows
                    .iter()
                    .filter_map(|row| {
                        row.get(field).and_then(|v| match v {
                            Value::Number(n) => Some(*n),
                            _ => None,
                        })
                    })
                    .collect();
                let sum: Option<f64> = if values.is_empty() {
                    None
                } else {
                    Some(values.iter().sum::<f64>())
                };
                result.sum = sum;
            }
            Aggregate::Avg(field) => {
                let values: Vec<f64> = rows
                    .iter()
                    .filter_map(|row| {
                        row.get(field).and_then(|v| match v {
                            Value::Number(n) => Some(*n),
                            _ => None,
                        })
                    })
                    .collect();

                if !values.is_empty() {
                    let avg = values.iter().sum::<f64>() / values.len() as f64;
                    result.avg = Some(avg);
                }
            }
            Aggregate::Min(field) => {
                let min = rows
                    .iter()
                    .filter_map(|row| {
                        row.get(field).and_then(|v| match v {
                            Value::Number(n) => Some(*n),
                            _ => None,
                        })
                    })
                    .fold(None::<f64>, |acc, val| {
                        Some(acc.map_or(val, |a| a.min(val)))
                    });
                result.min = min;
            }
            Aggregate::Max(field) => {
                let max = rows
                    .iter()
                    .filter_map(|row| {
                        row.get(field).and_then(|v| match v {
                            Value::Number(n) => Some(*n),
                            _ => None,
                        })
                    })
                    .fold(None::<f64>, |acc, val| {
                        Some(acc.map_or(val, |a| a.max(val)))
                    });
                result.max = max;
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::row::{Row, RowKind};

    #[test]
    fn test_count_aggregate() {
        let mut rows = Vec::new();
        for i in 1..=5 {
            let mut row = Row::new(RowKind::LIN);
            row.insert("qty", Value::Number(i as f64));
            rows.push(row);
        }

        let agg = Aggregate::Count("qty".to_string());
        let result = execute_aggregates(&rows, &[agg]);

        assert_eq!(result.count, Some(5));
    }

    #[test]
    fn test_sum_aggregate() {
        let mut rows = Vec::new();
        for i in 1..=5 {
            let mut row = Row::new(RowKind::LIN);
            row.insert("qty", Value::Number(i as f64));
            rows.push(row);
        }

        let agg = Aggregate::Sum("qty".to_string());
        let result = execute_aggregates(&rows, &[agg]);

        assert_eq!(result.sum, Some(15.0));
    }

    #[test]
    fn test_avg_aggregate() {
        let mut rows = Vec::new();
        for i in [10, 20, 30] {
            let mut row = Row::new(RowKind::LIN);
            row.insert("qty", Value::Number(i as f64));
            rows.push(row);
        }

        let agg = Aggregate::Avg("qty".to_string());
        let result = execute_aggregates(&rows, &[agg]);

        assert_eq!(result.avg, Some(20.0));
    }

    #[test]
    fn test_min_max_aggregate() {
        let mut rows = Vec::new();
        for i in [10, 50, 30, 70, 20] {
            let mut row = Row::new(RowKind::LIN);
            row.insert("qty", Value::Number(i as f64));
            rows.push(row);
        }

        let aggs = vec![
            Aggregate::Min("qty".to_string()),
            Aggregate::Max("qty".to_string()),
        ];
        let result = execute_aggregates(&rows, &aggs);

        assert_eq!(result.min, Some(10.0));
        assert_eq!(result.max, Some(70.0));
    }
}
