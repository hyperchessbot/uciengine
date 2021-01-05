import re


def read_file(path):
    with open(path) as file:
        return file.read()


def write_file(path, content):
    with open(path, "w") as file:
        file.write(content)


def decorate(text, pre):
    return "\n".join([pre + line for line in text.split("\n")])


example = read_file("src/example.rs")
parts = read_file("src/lib.rs").split("// lib\n")

lib = decorate(example, "//!") + "\n\n// lib\n" + parts[1]
# print(lib)
write_file("src/lib.rs", lib)

parts = re.split("# Usage\n|# Logging\n", read_file("ReadMe.md"))

readme = parts[0] + "# Usage\n\n```rust\n" + example
readme = readme + "```\n\n# Logging\n" + parts[2]
# print(readme)
write_file("ReadMe.md", readme)
