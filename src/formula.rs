use std::fmt::{Debug, Formatter};
use std::ops::Index;

pub type VarIndex = usize; // from 0

#[derive(Clone)]
pub struct Lit {
    // Literal (Boolean variable or its negation)
    pub var_idx: VarIndex,
    pub polarity: bool,
}

#[derive(Clone)]
pub struct Clause {
    // Disjunction (OR) of literals
    pub lits: Vec<Lit>,
}

pub struct Formula {
    // Conjunction (AND) of clauses
    pub clauses: Vec<Clause>,
    pub num_vars: usize,
}

pub enum ClauseStatus {
    Satisfied,   // true
    Conflicting, // false
    Unit(Lit),   // exactly one literal undetermined
    Unresolved,  // other cases
}

impl Lit {
    pub fn negate(&self) -> Self {
        Self {
            var_idx: self.var_idx,
            polarity: !self.polarity,
        }
    }

    pub fn evaluate(
        &self,
        assignment: &impl Index<VarIndex, Output = Option<bool>>,
    ) -> Option<bool> {
        assignment[self.var_idx].map(|b| match self.polarity {
            true => b,
            false => !b,
        })
    }
}

impl Clause {
    pub fn check(&self, assignment: &impl Index<VarIndex, Output = Option<bool>>) -> ClauseStatus {
        let mut num_undetermined_lits = 0;
        let mut last_undetermined_lit: Option<Lit> = None;
        for lit in self.lits.iter() {
            match lit.evaluate(assignment) {
                Some(true) => return ClauseStatus::Satisfied,
                Some(false) => {}
                None => {
                    num_undetermined_lits += 1;
                    last_undetermined_lit = Some(lit.clone());
                }
            }
        }
        match num_undetermined_lits {
            0 => ClauseStatus::Conflicting,
            1 => ClauseStatus::Unit(last_undetermined_lit.unwrap()),
            _ => ClauseStatus::Unresolved,
        }
    }
}

impl Formula {
    pub fn from_dimacs(clauses: impl IntoIterator<Item = impl IntoIterator<Item = isize>>) -> Self {
        let clauses: Vec<Clause> = clauses
            .into_iter()
            .map(|lits| Clause {
                lits: lits
                    .into_iter()
                    .map(|i| Lit {
                        var_idx: i.unsigned_abs() - 1,
                        polarity: i.is_positive(),
                    })
                    .collect(),
            })
            .collect();
        let num_vars = clauses
            .iter()
            .flat_map(|c| c.lits.iter())
            .map(|l| l.var_idx + 1)
            .max()
            .unwrap_or(0);
        Self { clauses, num_vars }
    }
}

impl Debug for Lit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            match self.polarity {
                true => "+",
                false => "-",
            },
            self.var_idx + 1
        )
    }
}

impl Debug for Clause {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "( ")?;
        for lit in self.lits.iter() {
            write!(f, "{lit:?} ")?;
        }
        write!(f, ")")
    }
}

impl Debug for Formula {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[ ")?;
        for clause in self.clauses.iter() {
            write!(f, "{clause:?} ")?;
        }
        write!(f, "]")
    }
}
