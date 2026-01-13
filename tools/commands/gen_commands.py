#!/usr/bin/env python3

import re

import yaml


def natural_sort_key(s):
    a = re.split(r"(\d+)", s)

    text = "".join([x for x in a if not x.isdigit()])
    numbers = [int(x) for x in a if x.isdigit()]

    n = numbers[0] if numbers else -1

    return (text[:6], n, text[6:])


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

    for k in sorted(data, key=natural_sort_key):
        # print(natural_sort_key(k))
        v = data[k]
        if "key" in v:
            shortcut = f'Some("{v["key"]}")'
        else:
            shortcut = "None"
        command = v["command"]
        f.write(
            f"""    Command {{
        name: "{k}",
        shortcut: {shortcut},
        action: |w| w.{command},
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
