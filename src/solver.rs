use crate::formula::{ClauseStatus, Formula, Lit};
use std::fmt::{Debug, Formatter};

struct Decision {
    lit: Lit,
}

struct Trail {
    decisions: Vec<Decision>,
    num_vars: usize,
}

impl Trail {
    fn new(num_vars: usize) -> Self {
        Self {
            num_vars,
            decisions: vec![],
        }
    }

    fn num_decisions(&self) -> usize {
        self.decisions.len()
    }

    fn num_decisions_until_var_set(&self, lit: &Lit) -> Option<usize> {
        self.decisions
            .iter()
            .position(|Decision { lit: l }| l.var_idx == lit.var_idx)
            .map(|i| i + 1)
    }

    fn find_last_positive_decision(&self) -> Option<&Decision> {
        self.decisions
            .iter()
            .rfind(|decision| decision.lit.polarity == true)
    }

    fn add_decision(&mut self, decision: Decision) {
        self.decisions.push(decision)
    }

    fn backjump(&mut self, num_decisions_to_keep: usize) {
        self.decisions.truncate(num_decisions_to_keep);
    }

    fn assignment(&self) -> Vec<Option<bool>> {
        let mut assignment = vec![None; self.num_vars];
        for Decision { lit } in &self.decisions {
            assignment[lit.var_idx] = Some(lit.polarity)
        }
        assignment
    }
}

pub enum SatResult {
    Sat { assignment: Vec<Option<bool>> },
    Unsat,
}

pub fn solve(formula: &Formula) -> SatResult {
    let mut trail = Trail::new(formula.num_vars);
    'main: loop {
        println!(" Trail: {trail:?}");
        for clause in formula.clauses.iter() {
            match clause.check(&trail.assignment()) {
                ClauseStatus::Satisfied | ClauseStatus::Unresolved => continue,
                ClauseStatus::Conflicting => {
                    println!("  Conflicting clause: {clause:?}");
                    match trail.find_last_positive_decision() {
                        Some(positive_decision) => {
                            // We've learned that var := true doesn't work
                            // Try var := false next
                            let num_decisions_until_positive_decision = trail
                                .num_decisions_until_var_set(&positive_decision.lit)
                                .unwrap();
                            let flipped_decision = Decision {
                                lit: positive_decision.lit.negate(),
                            };

                            trail.backjump(num_decisions_until_positive_decision - 1);
                            trail.add_decision(flipped_decision);
                            continue 'main;
                        }
                        None => return SatResult::Unsat,
                    }
                }
            }
        }

        if trail.num_decisions() < formula.num_vars {
            // Decide next var := true
            trail.add_decision(Decision {
                lit: Lit {
                    var_idx: trail.num_decisions(),
                    polarity: true,
                },
            });
        } else {
            return SatResult::Sat {
                assignment: trail.assignment(),
            };
        }
    }
}

impl Debug for Trail {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[ ")?;
        for Decision { lit } in self.decisions.iter() {
            write!(f, "{lit:?} ")?;
        }
        write!(f, "]")
    }
}

#[cfg(test)]
mod tests {
    use crate::formula::Formula;
    use crate::solver::{SatResult, solve};

    fn assert_sat(clauses: impl IntoIterator<Item = impl IntoIterator<Item = isize>>) {
        let formula = Formula::from_dimacs(clauses);
        println!("\nSolving SAT formula: {formula:?}");
        assert!(matches!(solve(&formula), SatResult::Sat { .. }))
    }

    fn assert_unsat(clauses: impl IntoIterator<Item = impl IntoIterator<Item = isize>>) {
        let formula = Formula::from_dimacs(clauses);
        println!("\nSolving UNSAT formula: {formula:?}");
        assert!(matches!(solve(&formula), SatResult::Unsat))
    }

    #[test]
    fn test_small_sat_instances() {
        assert_sat([[1]]);
        assert_sat([[-1]]);
        assert_sat([[1], [-2]]);
        assert_sat([vec![1], vec![2, -3]]);
        assert_sat([vec![-1], vec![1, 2]]);
        assert_sat([vec![1], vec![-1, 2], vec![-2, 3]]);
        assert_sat([
            vec![1],
            vec![1, -2],
            vec![-1, 2, -3],
            vec![-2, 3],
            vec![-1, -2, -3],
        ]);
    }

    #[test]
    fn test_small_unsat_instances() {
        assert_unsat([[]]);
        assert_unsat([[1], [-1]]);
        assert_unsat([vec![1], vec![-1, 2], vec![-1, -2]]);
        assert_unsat([
            vec![1, -2, 3],
            vec![-1, 2],
            vec![-2, -3],
            vec![1, 2, -3],
            vec![1, 3],
            vec![-1, -2, 3],
        ]);
        assert_unsat([
            vec![1, -2],
            vec![1, -3],
            vec![1, -8],
            vec![1, -7],
            vec![-4, 3],
            vec![-5, -6],
            vec![-2, -3, 4],
            vec![-4, 6, 9],
            vec![-3, -7, 8],
            vec![1, 8],
            vec![1, 7],
            vec![4, 5, -1],
            vec![4, -6, -1],
            vec![-6, -4, -1],
            vec![6, -7, -1],
            vec![-7, 6, -1],
            vec![7, 6, -1],
        ]);
    }
}
