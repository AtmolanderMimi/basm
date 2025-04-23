# Preprocessor script to programatically generate flag tables of `basm run` and `basm compile`
# by parsing the cli. Replaces `{{#custom run-flags}}` and `{{#custom compile-flags}}`.
# I would have wanted to write this in rust too, but i don't want to deal with compiling it every time.
# (I tried and mdbook doesn't have a great way to include non-preprocessor build scripts)
# So, python will suffice as a platform agnostic, non-compiled option.

import json
import sys
import subprocess
import re
from collections.abc import Mapping
from collections.abc import Iterable

# adds markdown "`" arround the string
def code_snippetise(string: str) -> str:
    return "`"+string+"`"

# applies the `function` to all the fields with the name `field` in the json dictionary
def apply_to_recusively(dictionary, field: str, function):
    # we don't actually do recusion,
    # because python doesn't have a big enough max callback
    # instead we just use a work queue and an semi-infinite loop
    items = [dictionary]

    # loops while there are still items in items
    while True:
        try:
            item = items.pop()
        except:
            break

        # if the item is a dictionary
        if isinstance(item, Mapping):
            for key, value in item.items():
                if key == field:
                    # this single line makes me remember why i didn't go back to python after learning rust
                    # python is not explicit about what is a copy and what is not,
                    # in this case simply writing `value = function(value)` would do nothing
                    # This is also the case for any attempted mutability in "function",
                    # "function" can't mutate strings passed into it apparently
                    item[key] = function(value)
                if isinstance(value, (Iterable, Mapping)) and not isinstance(value, str):
                    items.append(value)

        # if the item is a list
        elif isinstance(item, Iterable):
            for value in item:
                if isinstance(value, (Iterable, Mapping)) and not isinstance(value, str):
                    items.append(value)

# replaces the placeholders with the flag tables in the string
def replace_flag(string: str) -> str:
    string = string.replace(r"{{#custom run-flags}}", run_flag_table)
    string = string.replace(r"{{#custom compile-flags}}", compile_flag_table)

    return string

# parses a line for (shorthand flag, longhand flag, description)
# all string return values are optional if the match is empty
def parse_flag_line(flag_line: str) -> (str, str, str):
    flag_line = flag_line.strip()

    # group 0: entire match (the whole line)
    # group 1: shorthand (with -)
    # group 2: longhand  (with --)
    # group 3: definition
    match = re.search(r"(?:(-[^ ]*), )?(?:(--[^ ]*(?: <[\d\D]*>)?))?[ ]*([\d\D]*)", flag_line)

    return (match.group(1), match.group(2), match.group(3))

# creates a markdown table from an array of arrays
# treats the first row as a title row
# all the row must be of same lenght
def markdown_table_from_2d_array(array: [[str]]) -> str:
    out = ""

    # makes the header
    header_cells = array.pop(0)
    for head in header_cells:
        out += head+' | '
    out += "\n"
    for _ in header_cells:
        out += "--- | "
    out += "\n"

    # makes the rest of the rows
    for row in array:
        for element in row:
            out += element+" | "
        out += "\n"

    return out

# gets and formats the strings of the given basm subcommand
def generate_flag_table(subcommand: str) -> str:
    output = subprocess.run(["cargo", "run", "--", subcommand, "-h"], capture_output=True, text=True)
    
    flag_lines = re.search(r"Options:\n([\d\D]*$)", output.stdout.strip()).group(1)

    # we add the header row preemptively
    grid = [["Shorthand", "Longhand", "Description"]]

    # build the grid row by row
    for line in flag_lines.splitlines():
        row = []

        (shorthand, longhand, description) = parse_flag_line(line)
        if shorthand is None:
            row.append("")
        else:
            row.append(code_snippetise(shorthand))
        
        if longhand is None:
            row.append("")
        else:
            row.append(code_snippetise(longhand))

        row.append(description)
        grid.append(row)

    return markdown_table_from_2d_array(grid)

if __name__ == '__main__':
    # globals, we don't want to recompute them every time
    run_flag_table = generate_flag_table("run")
    compile_flag_table = generate_flag_table("compile")


    # mdbook runs this program to check if we support certain formats
    # before actually running it without arguments,
    # we don't care about the format, but we don't want to run twice
    if len(sys.argv) > 1: # we check if we received any argument
        if sys.argv[1] == "supports": 
            sys.exit(0)

    # mdbook gives us the book via stdin
    context, book = json.load(sys.stdin)

    # DEBUG
    #f = open("that-json-from-mdbook.txt", "w")
    #f.write(json.dumps(context, indent=1))
    #f.write("\n")
    #f.write(json.dumps(book, indent=1))

    # replace the flag placeholders in content fields
    apply_to_recusively(book, "content", replace_flag)
            
    # mdbook wants its book back now
    print(json.dumps(book))