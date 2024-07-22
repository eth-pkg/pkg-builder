name = "hello-world-python"
architecture = "any"
summary = """Example Package
This is a short description of the package. It should provide a brief summary
of what the package does and its purpose. The short description should be
limited to a single line."""
conflicts = []
recommends = []
provides = []
suggests = []
depends = ["python3"]
add_files = ["dist/hello_world /usr/bin/hello_world"]
add_manpages = []
long_doc = """
Example Package
 This is a short description of the package. It should provide a brief summary
 of what the package does and its purpose. The short description should be
 limited to a single line.
 Long Description:
  Example description. If not provided, lintian will fail.
"""