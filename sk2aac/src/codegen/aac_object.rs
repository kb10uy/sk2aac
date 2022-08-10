use crate::codegen::CodeWriter;

use std::{
    io::{prelude::*, Result as IoResult},
    iter::{once, repeat},
};

/// Emits piece of AAC code.
pub trait AacObject {
    fn write_into<W: Write>(&self, w: &mut CodeWriter<W>) -> IoResult<()>;
}

/// `var renderer = ...`
pub struct RendererFetch<'a>(&'a str);

impl<'a> RendererFetch<'a> {
    pub fn new(name: &'a str) -> Self {
        RendererFetch(name)
    }
}

impl<'a> AacObject for RendererFetch<'a> {
    fn write_into<W: Write>(&self, w: &mut CodeWriter<W>) -> IoResult<()> {
        let object_name = self.0;

        w.write_yield(|w| {
            write!(
                w,
                r#"var renderer = (SkinnedMeshRenderer) gameObject.transform.Find("{object_name}").GetComponent<SkinnedMeshRenderer>();"#
            )
        })
    }
}

/// `var parameter = ...`
pub enum ParameterDefinition<'a> {
    Bool(&'a str),
    Integer(&'a str),
}

impl<'a> ParameterDefinition<'a> {
    pub const PARAMETER_VARNAME: &'static str = "parameter";
}

impl<'a> AacObject for ParameterDefinition<'a> {
    fn write_into<W: Write>(&self, w: &mut CodeWriter<W>) -> IoResult<()> {
        let param_name = Self::PARAMETER_VARNAME;
        match self {
            ParameterDefinition::Bool(p) => w.write(format_args!(
                r#"var {param_name} = layer.BoolParameter("{p}");"#
            )),
            ParameterDefinition::Integer(p) => w.write(format_args!(
                r#"var {param_name} = layer.IntParameter("{p}");"#
            )),
        }
    }
}

/// `var state = ...`
pub struct StateDefinition<'a> {
    state_var: &'a str,
    state_name: &'a str,
    blend_shapes: &'a [(&'a str, f64)],
    renderer: &'a str,
    right_of: Option<String>,
    indented: bool,
}

impl<'a> StateDefinition<'a> {
    pub fn new(state_var: &'a str, state_name: &'a str) -> Self {
        StateDefinition {
            state_var,
            state_name,
            blend_shapes: &[],
            renderer: "renderer",
            right_of: None,
            indented: false,
        }
    }

    pub fn right_of(mut self, state_name: &str) -> Self {
        self.right_of = Some(state_name.into());
        self
    }

    pub fn blend_shapes(mut self, items: &'a [(&'a str, f64)]) -> Self {
        self.blend_shapes = items;
        self
    }

    pub fn indented(mut self) -> Self {
        self.indented = true;
        self
    }
}

impl<'a> AacObject for StateDefinition<'a> {
    fn write_into<W: Write>(&self, w: &mut CodeWriter<W>) -> IoResult<()> {
        let StateDefinition {
            state_var,
            state_name,
            renderer,
            ..
        } = &self;

        if self.indented {
            w.write_yield(|w| {
                write!(w, r#"var {state_var} = layer.NewState("{state_name}")"#)?;
                if let Some(ro) = &self.right_of {
                    write!(w, r#".RightOf({ro})"#)?;
                }
                write!(w, r#".WithAnimation("#)
            })?;

            w.with_indent(|mut b| {
                b.write(r#"aac.NewClip()"#)?;
                b.with_indent(|mut b| {
                    for (name, value) in self.blend_shapes {
                        b.write(format_args!(
                            r#".BlendShape({renderer}, "{name}", {value:.1}f)"#
                        ))?;
                    }
                    Ok(())
                })
            })?;

            w.write(r#");"#)
        } else {
            w.write_yield(|w| {
                write!(w, r#"var {state_var} = layer.NewState("{state_name}")"#)?;
                if let Some(ro) = &self.right_of {
                    write!(w, r#".RightOf({ro})"#)?;
                }
                write!(w, r#".WithAnimation(aac.NewClip()"#)?;
                for (name, value) in self.blend_shapes {
                    write!(w, r#".BlendShape({renderer}, "{name}", {value:.1}f)"#)?;
                }
                write!(w, r#");"#)
            })
        }
    }
}

/// `state.TransitionTo()...`
pub struct Transition<'a> {
    from: Option<&'a str>,
    to: Option<&'a str>,
    condition: Option<Cond<'a>>,
}

impl<'a> Transition<'a> {
    pub fn new(from: &'a str, to: &'a str) -> Self {
        Transition {
            from: Some(from),
            to: Some(to),
            condition: None,
        }
    }

    pub fn exits(from: &'a str) -> Self {
        Transition {
            from: Some(from),
            to: None,
            condition: None,
        }
    }

    pub fn cond(mut self, condition: Cond<'a>) -> Self {
        if condition.is_valid() {
            self.condition = Some(condition);
        }
        self
    }
}

impl<'a> AacObject for Transition<'a> {
    fn write_into<W: Write>(&self, w: &mut CodeWriter<W>) -> IoResult<()> {
        let condition = match &self.condition {
            Some(c) => c,
            None => return Ok(()),
        };

        w.write_yield(|w| {
            match (self.from, self.to) {
                (Some(f), Some(t)) => write!(w, r#"{f}.TransitionsTo({t})"#)?,
                (Some(f), None) => write!(w, r#"{f}.Exits()"#)?,
                _ => unreachable!("Invalid transition"),
            }
            condition.write(w)?;
            write!(w, r#";"#)
        })
    }
}

pub enum Expr<'a> {
    IntEqual(&'a str, usize),
    IntNotEqual(&'a str, usize),
    IsTrue(&'a str),
    IsFalse(&'a str),
}

impl<'a> Expr<'a> {
    fn write<W: Write>(&self, w: &mut W) -> IoResult<()> {
        match self {
            Expr::IntEqual(p, v) => write!(w, r#"{p}.IsEqualTo({v})"#),
            Expr::IntNotEqual(p, v) => write!(w, r#"{p}.IsNotEqualTo({v})"#),
            Expr::IsTrue(p) => write!(w, r#"{p}.IsTrue()"#),
            Expr::IsFalse(p) => write!(w, r#"{p}.IsFalse()"#),
        }
    }
}

pub enum Cond<'a> {
    Or(Vec<Cond<'a>>),
    And(Vec<Cond<'a>>),
    Term(Expr<'a>),
}

impl<'a> Cond<'a> {
    pub fn is_valid(&self) -> bool {
        match self {
            Cond::Or(_) => self.is_valid_or(),
            Cond::And(_) => self.is_valid_and(),
            Cond::Term(_) => true,
        }
    }

    fn is_valid_and(&self) -> bool {
        match self {
            Cond::And(terms) => terms.iter().all(|t| matches!(t, Cond::Term(_))),
            _ => false,
        }
    }

    fn is_valid_or(&self) -> bool {
        match self {
            Cond::Or(terms) => terms.iter().all(|t| t.is_valid_and()),
            _ => false,
        }
    }

    fn write<W: Write>(&self, w: &mut W) -> IoResult<()> {
        match self {
            Cond::Or(and_clauses) => {
                let or_splits = once("").chain(repeat(".Or()"));
                for (and_clause, or) in and_clauses.into_iter().zip(or_splits) {
                    write!(w, r#"{or}"#)?;
                    and_clause.write(w)?;
                }
            }
            Cond::And(terms) => {
                let method_names = once("When").chain(repeat("And"));
                for (term, method) in terms.into_iter().zip(method_names) {
                    let term = match term {
                        Cond::Term(t) => t,
                        _ => unreachable!("Should be validated"),
                    };
                    write!(w, r#".{method}("#)?;
                    term.write(w)?;
                    write!(w, r#")"#)?;
                }
            }
            Cond::Term(t) => {
                write!(w, r#".When("#)?;
                t.write(w)?;
                write!(w, r#")"#)?;
            }
        }
        Ok(())
    }
}
