load("@rules_rust//rust:defs.bzl", "rust_binary")
load("@io_bazel_rules_docker//lang:image.bzl", "app_layer")

rust_binary(
    name = "bot",
    srcs = ["bot.rs"],
    deps = [
        "@crate_index//:tokio",
        "@crate_index//:serenity",
    ],
)

app_layer(
    name = "bot_image",
    base = "@docker_cc_debian11_base//image",
    binary = ":bot",
)
