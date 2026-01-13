#!/usr/bin/env python3

import yaml

with open("commands.yaml") as stream:
    data = yaml.safe_load(stream)


with open("../../src/window/imp/commands.rs", "w") as f:
    f.write(
        """use crate::window::imp::MViewWindowImp;

#[derive(Clone)]
pub struct Command {
    pub name: &'static str,
    pub shortcut: Option<&'static str>,
    pub action: fn(&MViewWindowImp),
}

pub const COMMANDS: &[Command] = &[
"""
    )

    for k, v in data.items():
        if "key" in v:
            shortcut = f'Some("{v["key"]}")'
        else:
            shortcut = "None"
        f.write(
            f"""    Command {{
        name: "{k}",
        shortcut: {shortcut},
        action: |w| w.adjust_filter(),
    }},
"""
        )

        # print(k, ":", v)

    f.write(
        """];
"""
    )
# test = {
#     "1": ["a", "b"],
#     "2": ["a"],
#     "3": [],
# }

# print(yaml.dump(test))
