name = "hello-world-dotnet"
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
add_files = ["bin /usr/lib/hello-world-dotnet",
    # you will need obj first time building
   #"obj /usr/lib/hello-world-dotnet"
]
add_links = ["/usr/lib/hello-world-dotnet/bin/hello_world /usr/bin/hello-world"]
add_manpages = []
long_doc = """
Example Package
 This is a short description of the package. It should provide a brief summary
 of what the package does and its purpose. The short description should be
 limited to a single line.
 Long Description:
  Example description. If not provided, lintian will fail.
"""