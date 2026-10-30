use crate::formula::{Clause, ClauseStatus, Formula, Lit, VarIndex, resolve};
use std::fmt::{Debug, Formatter};

struct Decision {
    lit: Lit,
}

struct Propagation {
    lit: Lit,
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

    fn num_decisions_until_var_set(&self, lit: &Lit) -> Option<usize> {
        self.levels
            .iter()
            .position(|level| level.lits().any(|l| l.var_idx == lit.var_idx))
    }

    fn propagations(&self) -> impl DoubleEndedIterator<Item = &Propagation> {
        self.levels
            .iter()
            .flat_map(|level| level.propagations.iter())
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

fn derive_learned_clause(conflict_clause: &Clause, trail: &Trail) -> Clause {
    println!("  Deriving learned clause:");
    let mut learned_clause = conflict_clause.clone();

    // Resolve until the clause has exactly one literal set in the last level
    // (or the clause becomes empty)
    loop {
        let mut num_decisions_until_vars_set: Vec<usize> = learned_clause
            .lits
            .iter()
            .map(|lit| trail.num_decisions_until_var_set(lit).unwrap())
            .collect();
        num_decisions_until_vars_set.sort();
        if !matches!(num_decisions_until_vars_set[..], [.., i, j] if i == j) {
            break;
        }
        // If two literals were set in the same level, at least one must be a propagation
        // (since each level has at most one decision)
        let propagation = trail
            .propagations()
            .rev()
            .filter(|p| learned_clause.lits.contains(&p.lit.negate()))
            .next()
            .unwrap();
        learned_clause = resolve(&propagation.reason, &propagation.lit, &learned_clause);
        println!(
            "    [{:?}] reason: {:?} -> {:?}",
            propagation.lit, propagation.reason, learned_clause,
        );
    }

    learned_clause
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

                    let learned_clause = derive_learned_clause(clause, &trail);
                    println!("  Learned clause: {learned_clause:?}");

                    let mut num_decisions_until_vars_set: Vec<usize> = learned_clause
                        .lits
                        .iter()
                        .map(|lit| trail.num_decisions_until_var_set(lit).unwrap())
                        .collect();
                    num_decisions_until_vars_set.sort();
                    let num_decisions_until_clause_becomes_unit =
                        match num_decisions_until_vars_set[..] {
                            [i] if i > 0 => 0,
                            [.., i, j] if i < j => i,
                            _ => return SatResult::Unsat,
                        };

                    trail.backjump(num_decisions_until_clause_becomes_unit);
                    clauses.push(learned_clause);
                    continue 'main;
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
