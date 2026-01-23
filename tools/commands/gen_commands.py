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
        """// MView6 -- High-performance PDF and photo viewer built with Rust and GTK4
//
// Copyright (c) 2024-2025 Martin van der Werff <github (at) newinnovations.nl>
//
// This file is part of MView6.
//
// MView6 is free software: you can redistribute it and/or modify it under the terms of
// the GNU Affero General Public License as published by the Free Software Foundation, either
// version 3 of the License, or (at your option) any later version.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR
// IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND
// FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY
// DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR
// BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT,
// STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

        use crate::window::imp::MViewWindowImp;

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
