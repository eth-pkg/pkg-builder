name = "hello-world-java"
architecture = "any"
summary = """Example Package
This is a short description of the package. It should provide a brief summary
of what the package does and its purpose. The short description should be
limited to a single line."""
conflicts = []
recommends = []
provides = []
suggests = []
depends = []
add_files = [
  "bin /usr/lib/hello-world-java",
  "debian/hello-world.sh /usr/lib/hello-world-java",
]
add_links=["/usr/lib/hello-world-java/hello-world.sh /usr/bin/hello-world"]
add_manpages = []
long_doc = """
Example Package
 This is a short description of the package. It should provide a brief summary
 of what the package does and its purpose. The short description should be
 limited to a single line.
 Long Description:
  Example description. If not provided, lintian will fail.
"""