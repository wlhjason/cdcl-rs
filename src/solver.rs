use crate::formula::{Clause, ClauseStatus, Formula, Lit, VarIndex};
use std::fmt::{Debug, Formatter};

struct Decision {
    lit: Lit,
}

struct Propagation {
    lit: Lit,
    #[allow(unused)]
    reason: Clause,
}

struct Level {
    decision: Option<Decision>,
    propagations: Vec<Propagation>,
}

impl Level {
    fn lits(&self) -> impl Iterator<Item = &Lit> {
        self.decision
            .iter()
            .map(|d| &d.lit)
            .chain(self.propagations.iter().map(|p| &p.lit))
    }
}

struct Trail {
    levels: Vec<Level>,
    num_vars: usize,
}

impl Trail {
    fn new(num_vars: usize) -> Self {
        Self {
            levels: vec![Level {
                decision: None,
                propagations: vec![],
            }],
            num_vars,
        }
    }

    fn decisions(&self) -> impl DoubleEndedIterator<Item = &Decision> {
        self.levels.iter().flat_map(|level| level.decision.iter())
    }

    fn num_decisions_until_var_set(&self, lit: &Lit) -> Option<usize> {
        self.levels
            .iter()
            .position(|level| level.lits().any(|l| l.var_idx == lit.var_idx))
    }

    fn find_last_positive_decision(&self) -> Option<&Decision> {
        self.decisions()
            .rfind(|decision| decision.lit.polarity == true)
    }

    fn add_decision(&mut self, decision: Decision) {
        self.levels.push(Level {
            decision: Some(decision),
            propagations: vec![],
        })
    }

    fn add_propagation(&mut self, propagation: Propagation) {
        self.levels
            .last_mut()
            .unwrap()
            .propagations
            .push(propagation)
    }

    fn backjump(&mut self, num_decisions_to_keep: usize) {
        self.levels.truncate(1 + num_decisions_to_keep);
    }

    fn assignment(&self) -> Vec<Option<bool>> {
        let mut assignment = vec![None; self.num_vars];
        for lit in self.levels.iter().flat_map(|level| level.lits()) {
            assignment[lit.var_idx] = Some(lit.polarity)
        }
        assignment
    }

    fn unassigned_var_idx(&self) -> Option<VarIndex> {
        self.assignment().iter().position(|a| a.is_none())
    }
}

pub enum SatResult {
    Sat { assignment: Vec<Option<bool>> },
    Unsat,
}

pub fn solve(formula: &Formula) -> SatResult {
    let mut clauses = formula.clauses.clone();
    let mut trail = Trail::new(formula.num_vars);
    'main: loop {
        println!(" Trail: {trail:?}");
        for clause in clauses.iter() {
            match clause.check(&trail.assignment()) {
                ClauseStatus::Satisfied | ClauseStatus::Unresolved => continue,
                ClauseStatus::Unit(lit) => {
                    trail.add_propagation(Propagation {
                        lit,
                        reason: clause.clone(),
                    });
                    continue 'main;
                }
                ClauseStatus::Conflicting => {
                    println!("  Conflicting clause: {clause:?}");
                    match trail.find_last_positive_decision() {
                        Some(positive_decision) => {
                            // We've learned that var := true doesn't work
                            // Try var := false next
                            let num_decisions_until_positive_decision = trail
                                .num_decisions_until_var_set(&positive_decision.lit)
                                .unwrap();
                            let learned_clause = Clause {
                                lits: trail
                                    .decisions()
                                    .take(num_decisions_until_positive_decision + 1)
                                    .map(|Decision { lit }| lit.negate())
                                    .collect(),
                            };
                            println!("  Learned clause: {learned_clause:?}");

                            trail.backjump(num_decisions_until_positive_decision - 1);
                            clauses.push(learned_clause);
                            continue 'main;
                        }
                        None => return SatResult::Unsat,
                    }
                }
            }
        }

        if let Some(unassigned_var_idx) = trail.unassigned_var_idx() {
            // Decide next var := true
            trail.add_decision(Decision {
                lit: Lit {
                    var_idx: unassigned_var_idx,
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

impl Debug for Level {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[ ")?;
        if let Some(Decision { lit }) = &self.decision {
            write!(f, "D={lit:?} ")?;
        }
        for Propagation { lit, .. } in self.propagations.iter() {
            write!(f, "{lit:?} ")?;
        }
        write!(f, "]")
    }
}

impl Debug for Trail {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (i, level) in self.levels.iter().enumerate() {
            write!(f, "L{i}={level:?} ")?;
        }
        Ok(())
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
