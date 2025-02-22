use std::{collections::HashMap, time::Duration};

use simplexpr::{dynval::DynVal, SimplExpr};
use yuck::config::{
    script_var_definition::{PollScriptVar, ScriptVarDefinition, VarSource},
    var_definition::VarDefinition,
};

use crate::{config::system_stats::*, EwwPaths};
use eww_shared_util::VarName;

macro_rules! define_builtin_vars {
    ($interval:expr, $($name:literal => $fun:expr),*$(,)?) => {
        pub static INBUILT_VAR_NAMES: &[&'static str] = &[$($name),*];
        pub fn get_inbuilt_vars() -> HashMap<VarName, ScriptVarDefinition> {
            maplit::hashmap! {
                $(
                VarName::from($name) => ScriptVarDefinition::Poll(PollScriptVar {
                    name: VarName::from($name),
                    run_while_expr: SimplExpr::Literal(DynVal::from(true)),
                    run_while_var_refs: Vec::new(),
                    command: VarSource::Function($fun),
                    initial_value: None,
                    interval: $interval,
                    name_span: eww_shared_util::span::Span::DUMMY,
                })
                ),*
            }
        }
    }
}

define_builtin_vars! { Duration::new(2, 0),
    // @desc EWW_TEMPS - Heat of the components in Celcius
    // @prop { <name>: temperature }
    "EWW_TEMPS" => || Ok(DynVal::from(get_temperatures())),

    // @desc EWW_RAM - Information on ram and swap usage in kB.
    // @prop { total_mem, free_mem, total_swap, free_swap, available_mem, used_mem, used_mem_perc }
    "EWW_RAM" => || Ok(DynVal::from(get_ram())),

    // @desc EWW_DISK - Information on on all mounted partitions (Might report inaccurately on some filesystems, like btrfs)\nExample: `{EWW_DISK["/"]}`
    // @prop { <mount_point>: { name, total, free, used, used_perc } }
    "EWW_DISK" => || Ok(DynVal::from(get_disks())),

    // @desc EWW_BATTERY - Battery capacity in procent of the main battery
    // @prop { <name>: { capacity, status } }
    "EWW_BATTERY" => || Ok(DynVal::from(
        match get_battery_capacity() {
            Err(e) => {
                log::error!("Couldn't get the battery capacity: {:?}", e);
                "Error: Check `eww log` for more details".to_string()
            }
            Ok(o) => o,
        }
    )),

    // @desc EWW_CPU - Information on the CPU cores: frequency and usage (No MacOS support)
    // @prop { cores: [{ core, freq, usage }], avg }
    "EWW_CPU" => || Ok(DynVal::from(get_cpus())),

    // @desc EWW_NET - Bytes up/down on all interfaces
    // @prop { <name>: { up, down } }
    "EWW_NET" => || Ok(DynVal::from(net())),
}

macro_rules! define_magic_constants {
    ($eww_paths:ident, $($name:literal => $value:expr),*$(,)?) => {
        pub static MAGIC_CONSTANT_NAMES: &[&'static str] = &[$($name),*];
        pub fn get_magic_constants($eww_paths: &EwwPaths) -> HashMap<VarName, VarDefinition> {
            maplit::hashmap! {
                $(VarName::from($name) => VarDefinition {
                    name: VarName::from($name),
                    initial_value: $value,
                    span: eww_shared_util::span::Span::DUMMY
                }),*
            }
        }
    }
}
define_magic_constants! { eww_paths,
    // @desc EWW_CONFIG_DIR - Path to the eww configuration of the current process
    "EWW_CONFIG_DIR" => DynVal::from_string(eww_paths.get_config_dir().to_string_lossy().into_owned()),

    // @desc EWW_CMD - eww command running in the current configuration, useful in event handlers. I.e.: `:onclick "${EWW_CMD} update foo=bar"`
    "EWW_CMD" => DynVal::from_string(
        format!("\"{}\" --config \"{}\"",
            std::env::current_exe().map(|x| x.to_string_lossy().into_owned()).unwrap_or_else(|_| "eww".to_string()),
            eww_paths.get_config_dir().to_string_lossy().into_owned()
        )
    ),
    // @desc EWW_EXECUTABLE - Full path of the eww executable
    "EWW_EXECUTABLE" => DynVal::from_string(
        std::env::current_exe().map(|x| x.to_string_lossy().into_owned()).unwrap_or_else(|_| "eww".to_string()),
    ),
}
