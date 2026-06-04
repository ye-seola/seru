use crate::runtime::{Runtime, Value, utils::check_integer};

impl Runtime {
    pub fn register_builtin_functions(&mut self) {
        self.set("repeat", Value::Func(repeat));
        self.set("if", Value::Func(if_));
    }
}

fn repeat(args: Vec<Value>) -> anyhow::Result<Value> {
    if args.len() != 2 {
        anyhow::bail!("repeat required 2 args")
    }

    let target = args[0].clone();

    let repeat_cnt = args[1].clone().into_number()?;
    if !check_integer(repeat_cnt) || repeat_cnt < 0.0 {
        anyhow::bail!("repeat_cnt must be non-negative integer")
    }

    let repeat_cnt = repeat_cnt as usize;

    Ok(match target {
        Value::String(s) => Value::String(s.repeat(repeat_cnt)),
        Value::Array(values) => {
            let mut repeated = Vec::with_capacity(values.len() * repeat_cnt);

            for _ in 0..repeat_cnt {
                repeated.extend_from_slice(&values);
            }

            Value::Array(repeated)
        }
        v => anyhow::bail!("cannot repeat {:?}", v.ty()),
    })
}

fn if_(args: Vec<Value>) -> anyhow::Result<Value> {
    if args.len() != 3 {
        anyhow::bail!("if required 3 args")
    }

    let cond = args[0].is_truthy();
    if cond {
        Ok(args[1].clone())
    } else {
        Ok(args[2].clone())
    }
}
