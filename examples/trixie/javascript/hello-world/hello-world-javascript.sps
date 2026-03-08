name = "hello-world-javascript"
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
  "src /usr/lib/hello-world-javascript",
  #"node_modules /usr/lib/hello-world-javascript",
  "package.json /usr/lib/hello-world-javascript",
  "package-lock.json /usr/lib/hello-world-javascript",
  "debian/hello-world.sh /usr/lib/hello-world-javascript"
  ]
add_links=["/usr/lib/hello-world-javascript/hello-world.sh /usr/bin/hello-world"]
add_manpages = []
long_doc = """
Example Package
 This is a short description of the package. It should provide a brief summary
 of what the package does and its purpose. The short description should be
 limited to a single line.
 Long Description:
  Example description. If not provided, lintian will fail.
"""