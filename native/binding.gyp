{
  "targets": [
    {
      "target_name": "skclisp_native",
      "sources": ["binding.cc"],
      "cflags": ["-Wall", "-Wextra", "-O3"],
      "cflags_cc": ["-std=c++17", "-Wall", "-Wextra", "-O3"],
      "ldflags": ["-ldl"],
      "include_dirs": [
        "<!(node -p \"require('path').join(require('os').homedir(), '.node-gyp', require('semver').major(process.version), 'include')\")"
      ]
    }
  ]
}
